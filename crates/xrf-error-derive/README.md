# xrf-error-derive

Provides `#[derive(ErrorConstructors)]` for enums with message-based error variants.

## Generate constructors

```rust
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

The generated methods are associated constructors on the enum. They accept either borrowed text or an owned `String`.

## Attributes

Derive `xrf_error_derive::ErrorConstructors` on an enum and mark selected variants with `#[constructor]`. A variant
`Invalid { message: String }` gets `new_invalid_error(message: impl Into<String>)`. Use
`#[constructor("custom_name")]` to choose the method name; unmarked variants get no constructor.

Each marked variant must have exactly one named `message` field compatible with `String`. The macro generates
constructors only; it does not implement `Display`, `Error`, or conversions.

| Attribute                   | Method generated for `NotFound` |
| --------------------------- | ------------------------------- |
| `#[constructor]`            | `new_not_found_error`           |
| `#[constructor("missing")]` | `missing`                       |
| No attribute                | No method                       |

Apply display and error derives separately when the enum needs them. Unmarked variants may carry other shapes, as the
`Cancelled` variant above does.

## Diagnostics and checks

The derive rejects non-enum inputs, repeated constructor attributes, malformed custom names, and marked variants with
tuple, unit, or extra fields. Rust type checking also requires the generated `String` to fit the `message` field.
Choose names that do not collide with other generated or handwritten methods.

Run `cargo test --locked -p xrf-error-derive` from the repository root. Its compile tests cover accepted declarations
and rejected attribute or variant shapes.

See the [implementation](src/lib.rs), [compile tests](tests), and its use in
[XrfError](../xrf-error/src/error.rs).
