/*!

# `derive_is_enum_variant`

[![](https://docs.rs/derive_is_enum_variant/badge.svg)](https://docs.rs/derive_is_enum_variant/) [![](http://meritbadge.herokuapp.com/derive_is_enum_variant) ![](https://img.shields.io/crates/d/derive_is_enum_variant.png)](https://crates.io/crates/derive_is_enum_variant) [![Build Status](https://travis-ci.org/fitzgen/derive_is_enum_variant.png?branch=master)](https://travis-ci.org/fitzgen/derive_is_enum_variant)

Stop writing `pub is_whatever(&self) -> bool` for your `enum`s by hand -- it's a
pain! Just `#[derive(is_enum_variant)]` instead!

## Usage

Add `derive_is_enum_variant` to your crate's `Cargo.toml`:

```toml
[dependencies]
derive_is_enum_variant = "<insert-latest-version-here>"
```

And then add `#[derive(is_enum_variant)]` to your `enum` definitions:

```rust
#[macro_use]
extern crate derive_is_enum_variant;

#[derive(is_enum_variant)]
pub enum Pet {
    Doggo,
    Kitteh,
}

fn main() {
    let pet = Pet::Doggo;

    assert!(pet.is_doggo());
    assert!(!pet.is_kitteh());
}
```

### Customizing Predicate Names

By default, the predicates are named `is_snake_case_of_variant_name`. You can
use any name you want instead with `#[is_enum_variant(name = "..")]`:

```rust
# #[macro_use]
# extern crate derive_is_enum_variant;

#[derive(is_enum_variant)]
pub enum Pet {
    #[is_enum_variant(name = "is_real_good_boy")]
    Doggo,
    Kitteh,
}

# fn main() {
let pet = Pet::Doggo;
assert!(pet.is_real_good_boy());
# }
```

### Skipping Predicates for Certain Variants

If you don't want to generate a predicate for a certain variant, you can use
`#[is_enum_variant(skip)]`:

```rust
# #[macro_use]
# extern crate derive_is_enum_variant;

#[derive(is_enum_variant)]
pub enum Errors {
    Io(::std::io::Error),

    #[doc(hidden)]
    #[is_enum_variant(skip)]
    __NonExhaustive,
}

# fn main() {}
```

## License

Licensed under either of

  * Apache License, Version 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([`LICENSE-MIT`](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

See [CONTRIBUTING.md](./CONTRIBUTING.md) for hacking!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

*/

extern crate heck;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use heck::SnakeCase;
use proc_macro::TokenStream;

#[proc_macro_derive(is_enum_variant, attributes(is_enum_variant))]
pub fn derive_is_enum_variant(tokens: TokenStream) -> TokenStream {
    let source = tokens.to_string();

    let ast = syn::parse_derive_input(&source).expect("should parse input tokens into AST");

    let expanded = expand_derive_is_enum_variant(&ast);

    expanded
        .parse()
        .expect("should parse expanded output source into tokens")
}

enum PredicateConfig {
    None,
    Skip,
    Name(String),
}

impl PredicateConfig {
    fn join(self, meta: &syn::MetaItem) -> Self {
        match *meta {
            syn::MetaItem::Word(ref ident) if ident.to_string() == "skip" => match self {
                PredicateConfig::None | PredicateConfig::Skip => PredicateConfig::Skip,
                PredicateConfig::Name(_) => panic!(
                    "Cannot both `#[is_enum_variant(skip)]` and \
                     `#[is_enum_variant(name = \"..\")]`"
                ),
            },
            syn::MetaItem::NameValue(ref ident, syn::Lit::Str(ref s, _))
                if ident.to_string() == "name" =>
            {
                if !s.chars().all(|c| match c {
                    '_' | 'a'...'z' | 'A'...'Z' | '0'...'9' => true,
                    _ => false,
                }) {
                    panic!(
                        "#[is_enum_variant(name = \"..\")] must be provided \
                         a valid identifier"
                    )
                }
                match self {
                    PredicateConfig::None => PredicateConfig::Name(s.to_string()),
                    PredicateConfig::Skip => panic!(
                        "Cannot both `#[is_enum_variant(skip)]` and \
                         `#[is_enum_variant(name = \"..\")]`"
                    ),
                    PredicateConfig::Name(_) => panic!(
                        "Cannot provide more than one \
                         `#[is_enum_variant(name = \"..\")]`"
                    ),
                }
            }
            ref otherwise => panic!(
                "Unknown item inside `#[is_enum_variant(..)]`: {:?}",
                otherwise
            ),
        }
    }
}

impl<'a> From<&'a Vec<syn::Attribute>> for PredicateConfig {
    fn from(attrs: &'a Vec<syn::Attribute>) -> Self {
        let our_attr = attrs.iter().find(|a| a.name() == "is_enum_variant");
        our_attr.map_or(PredicateConfig::None, |attr| match attr.value {
            syn::MetaItem::List(_, ref metas) => metas
                .iter()
                .map(|m| match *m {
                    syn::NestedMetaItem::MetaItem(ref m) => m,
                    syn::NestedMetaItem::Literal(_) => panic!("Invalid #[is_enum_variant] item"),
                })
                .fold(PredicateConfig::None, PredicateConfig::join),
            _ => panic!(
                "#[is_enum_variant] must be used with name/value pairs, like \
                 #[is_enum_variant(name = \"..\")]"
            ),
        })
    }
}

fn expand_derive_is_enum_variant(ast: &syn::DeriveInput) -> quote::Tokens {
    let variants = match ast.body {
        syn::Body::Struct(_) => panic!("#[derive(is_enum_variant)] can only be used with enums"),
        syn::Body::Enum(ref variants) => variants,
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let predicates = variants.iter().map(
        |&syn::Variant {
             ref ident,
             ref data,
             ref attrs,
             ..
         }| {
            let cfg = attrs.into();
            if let PredicateConfig::Skip = cfg {
                return quote!{};
            }

            let variant_name = ident.to_string();
            let doc = format!("Is this `{}` a `{}`?", name, variant_name);

            let predicate_name = if let PredicateConfig::Name(name) = cfg {
                name
            } else {
                let mut name = String::from("is_");
                name.push_str(&variant_name.to_snake_case());
                name
            };
            let predicate_name = quote::Ident::new(predicate_name);

            let data_tokens = match *data {
                syn::VariantData::Struct(..) => quote! { { .. } },
                syn::VariantData::Tuple(..) => quote! { (..) },
                syn::VariantData::Unit => quote!{},
            };

            quote! {
                #[doc = #doc]
                #[inline]
                #[allow(unreachable_patterns)]
                #[allow(dead_code)]
                pub fn #predicate_name(&self) -> bool {
                    match *self {
                        #name :: #ident #data_tokens => true,
                        _ => false,
                    }
                }
            }
        },
    );

    quote! {
        /// # `enum` Variant Predicates
        impl #impl_generics #name #ty_generics #where_clause {
            #(
                #predicates
            )*
        }
    }
}
