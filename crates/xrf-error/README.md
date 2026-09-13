# xrf-error

Shared error and result types for XRF libraries and binaries.

## Return a typed error

```rust
use xrf_error::{XrfError, XrfResult};

fn require_section(name: &str) -> XrfResult<&str> {
  if name.is_empty() {
    return Err(XrfError::new_invalid_error("Section name is empty"));
  }

  Ok(name)
}

fn main() -> XrfResult {
  assert_eq!(require_section("actor")?, "actor");
  assert!(matches!(require_section(""), Err(XrfError::Invalid { .. })));

  Ok(())
}
```

## Results and constructors

Return `XrfResult<T>` for a fallible operation, or `XrfResult` for one returning no value. Create a domain error with
an `XrfError` constructor such as `new_invalid_error` or `new_parsing_error`; propagate supported conversions with `?`.

Message-only variants have generated constructors accepting `Into<String>`. Variants carrying structured context have
their own constructors or fields; retain that context instead of flattening the entire error into a string.

## Classify failures

| Variant          | Meaning for the caller                        |
| ---------------- | --------------------------------------------- |
| `Invalid`        | Input violates an operation's requirements    |
| `Parsing`        | Input cannot be parsed                        |
| `NotFound`       | The requested asset or value is absent        |
| `Io`             | Host I/O failed; inspect its `ErrorKind`      |
| `NotImplemented` | The requested operation has no implementation |
| `Cancelled`      | Cooperative cancellation was requested        |

Match variants when behavior depends on the cause. Display messages are for readers, not for classifying errors.
The conversions in `from.rs` determine how dependency errors map into this model; `?` only works where a conversion
exists. Wrapping every error as `Invalid` would discard distinctions such as a missing file versus a read failure.

## Serialization and checks

Errors serialize through Serde. The optional `typescript-bindings` feature adds frontend type metadata.

Run `cargo test --locked -p xrf-error` from the repository root. This crate defines errors; it does not log them, select
process exit codes, or decide how a UI displays a failure.

See the [variants and constructors](src/error.rs), [conversions](src/from.rs), and [result alias](src/types.rs).
