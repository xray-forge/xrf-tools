# xrf-error-derive

Provides `#[derive(ErrorConstructors)]`, which generates a constructor for each marked message-only variant of an
error enum. [`XrfError`](../xrf-error/src/xrf_error.rs) uses it for its `new_*_error` constructors; depend on `xrf-error`
rather than on this crate unless you are defining another error enum.

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

The example is not run as a doc test; [tests/ui/pass.rs](tests/ui/pass.rs) compiles the same shape. Each generated
method is a `pub` associated function taking `impl Into<String>`, so borrowed text and an owned
`String` both work.

| Attribute                   | Method generated for `NotFound` |
| --------------------------- | ------------------------------- |
| `#[constructor]`            | `new_not_found_error`           |
| `#[constructor("missing")]` | `missing`                       |
| No attribute                | No method                       |

The default name is `new_<variant in snake_case>_error`. A marked variant must have exactly one named field,
`message: String`; unmarked variants may take any shape, as `Cancelled` does above. A generic enum gets its constructors
in an `impl` over the same parameters and bounds. Each constructor is documented as the variant it builds. The derive
writes constructors only, so derive `Display`, `Error` and conversions separately, and pick custom names that do not
collide with handwritten methods.

## Diagnostics and checks

The derive rejects a non-enum input, a repeated `#[constructor]`, an attribute that is not `#[constructor]` or
`#[constructor("name")]`, a name that is not a Rust identifier, and a marked variant with tuple, unit or extra fields.

Run `cargo test --locked -p xrf-error-derive` from the repository root. Its [trybuild tests](tests/ui.rs) compile an
accepted declaration and check rejected ones against the `.stderr` snapshots. See the [implementation](src/lib.rs).
