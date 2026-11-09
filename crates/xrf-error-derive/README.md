# xrf-error-derive

Provides `#[derive(ErrorConstructors)]`, which generates a constructor for each marked message-only variant of an
error enum.

[`XrfError`](../xrf-error/src/xrf_error.rs) uses it for its `new_*_error` constructors. Depend on `xrf-error` instead,
unless you are defining another error enum.

## Generate constructors

```rust,ignore
use xrf_error_derive::ErrorConstructors;

#[derive(Debug, PartialEq, ErrorConstructors)]
enum InputError {
  #[constructor]
  Invalid {
    message: String,
  },
  #[constructor("missing")]
  NotFound {
    message: String,
  },
  Cancelled,
}

fn main() {
  assert_eq!(
    InputError::new_invalid_error("Empty name"),
    InputError::Invalid {
      message: "Empty name".to_owned()
    },
  );
  assert!(matches!(InputError::missing("No config"), InputError::NotFound { .. }));
}
```

The example is not run as a doc test; [tests/ui/pass.rs](tests/ui/pass.rs) compiles the same shape.

| Attribute                   | Method generated for `NotFound` |
| --------------------------- | ------------------------------- |
| `#[constructor]`            | `new_not_found_error`           |
| `#[constructor("missing")]` | `missing`                       |
| No attribute                | No method                       |

- The default name is `new_<variant in snake_case>_error`.
- Each method is a `pub` associated function taking `impl Into<String>`, so borrowed and owned text both work.
- A marked variant must have exactly one named field, `message: String`. Unmarked variants may have any shape, like
  `Cancelled` above.
- A generic enum gets its constructors in an `impl` with the same parameters and bounds.
- Each constructor is documented as the variant it builds.
- The derive writes constructors only. Derive `Display`, `Error` and conversions separately, and pick custom names that
  do not collide with handwritten methods.

## Compile errors

The derive rejects:

- input that is not an enum;
- a repeated `#[constructor]`;
- an attribute other than `#[constructor]` or `#[constructor("name")]`;
- a name that is not a Rust identifier;
- a marked variant with tuple, unit or extra fields.

## Checks

Run `cargo test --locked -p xrf-error-derive` from the repository root. Its [trybuild tests](tests/ui.rs) compile an
accepted declaration and check rejected ones against `.stderr` snapshots. See the [implementation](src/lib.rs).
