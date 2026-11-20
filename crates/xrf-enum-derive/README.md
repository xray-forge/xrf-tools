# xrf-enum-derive

Derives text and the variant list for enums whose variants carry no fields: how a variant reads and writes as text, and
every variant in order.

| Derive         | Generates                                                                                    |
| -------------- | -------------------------------------------------------------------------------------------- |
| `EnumDisplay`  | `Display`, writing each variant's `#[display("text")]`, or its name without one              |
| `EnumFromStr`  | `FromStr`, reading the same text back in any ASCII case; an unknown text is an `XrfError`    |
| `EnumVariants` | `const VARIANTS: &[Self]`, every variant in declaration order, kept complete by the compiler |

## Derive text and variants

```rust,ignore
use std::str::FromStr;

use xrf_enum_derive::{EnumDisplay, EnumFromStr, EnumVariants};

#[derive(Clone, Copy, Debug, PartialEq, EnumDisplay, EnumFromStr, EnumVariants)]
enum Language {
  #[display("eng")]
  English,
  #[display("ukr")]
  Ukrainian,
  Unknown,
}

fn main() {
  assert_eq!(Language::English.to_string(), "eng");
  assert_eq!(Language::from_str("UKR").ok(), Some(Language::Ukrainian));
  assert_eq!(Language::from_str("unknown").ok(), Some(Language::Unknown));
  assert_eq!(Language::VARIANTS.len(), 3);
}
```

The example is not run as a doc test; [tests/ui/pass.rs](tests/ui/pass.rs) compiles and runs the same shape.

- `EnumFromStr` reads what `EnumDisplay` writes, so every variant's text parses back to it. It matches the
  `#[display]` text, not the variant name, when the two differ. It fails with an `XrfError` parsing error naming the
  enum and the text, so the deriving crate depends on `xrf-error`.
- Derive only what the enum needs. An enum with a handwritten `FromStr` can still derive `EnumDisplay`.
- A struct whose `Display` formats its fields, such as `"{x},{y},{z}"`, writes the impl by hand; this crate covers
  fieldless enums only.

## Compile errors

Each derive rejects:

- input that is not an enum;
- a generic enum;
- a variant with fields;
- a repeated `#[display]`, or one that is not `#[display("text")]`.

## Checks

Run `cargo test --locked -p xrf-enum-derive` from the repository root. Its [trybuild tests](tests/ui.rs) run an
accepted declaration and check rejected ones against `.stderr` snapshots. See the [implementation](src/lib.rs).
