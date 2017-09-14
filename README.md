# derive_is_enum_variant

## `derive_is_enum_variant`

[![](https://docs.rs/derive_is_enum_variant/badge.svg)](https://docs.rs/derive_is_enum_variant/) [![](http://meritbadge.herokuapp.com/derive_is_enum_variant) ![](https://img.shields.io/crates/d/derive_is_enum_variant.png)](https://crates.io/crates/derive_is_enum_variant) [![Build Status](https://travis-ci.org/fitzgen/derive_is_enum_variant.png?branch=master)](https://travis-ci.org/fitzgen/derive_is_enum_variant)

Stop writing `pub is_whatever(&self) -> bool` for your `enum`s by hand -- it's a
pain! Just `#[derive(is_enum_variant)]` instead!

### Usage

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

#### Customizing Predicate Names

By default, the predicates are named `is_snake_case_of_variant_name`. You can
use any name you want instead with `#[is_enum_variant(name = "..")]`:

```rust

#[derive(is_enum_variant)]
pub enum Pet {
    #[is_enum_variant(name = "is_real_good_boy")]
    Doggo,
    Kitteh,
}

let pet = Pet::Doggo;
assert!(pet.is_real_good_boy());
```

#### Skipping Predicates for Certain Variants

If you don't want to generate a predicate for a certain variant, you can use
`#[is_enum_variant(skip)]`:

```rust

#[derive(is_enum_variant)]
pub enum Errors {
    Io(::std::io::Error),

    #[doc(hidden)]
    #[is_enum_variant(skip)]
    __NonExhaustive,
}

```

### License

Licensed under either of

  * Apache License, Version 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([`LICENSE-MIT`](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

See [CONTRIBUTING.md](./CONTRIBUTING.md) for hacking!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


License: Apache-2.0/MIT
