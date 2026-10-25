# xrf-error

The error and result types every XRF crate and binary shares: `XrfError`, one enum of failure kinds, and
`XrfResult<T = ()>`, an alias for `Result<T, XrfError>`. Format crates, the CLI and the desktop app's commands all
return them, so a failure keeps its kind from the reader that found it to the surface that reports it.

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

Message-only variants get a `new_<variant>_error(impl Into<String>)` constructor from
[xrf-error-derive](../xrf-error-derive/README.md). Variants with structured context have handwritten constructors:

- `new_io_error(message, kind)` and `new_chunk_not_ended_error(message, remaining)`;
- `new_ltx_parse_error(line, col, message)` and `new_ltx_scheme_error(section, field, message)`.

Keep that context rather than formatting it into a message.

## Choose a variant

Match on the variant when behavior depends on the cause; the `Display` text is for people.

| Variant                 | Meaning                                                                         |
| ----------------------- | ------------------------------------------------------------------------------- |
| `Invalid`               | Input or arguments violate an operation's requirements                          |
| `Parsing`               | Input cannot be parsed                                                          |
| `NotFound`              | A requested file, asset, section or value is absent                             |
| `Io`                    | Host I/O failed; `kind` keeps the `std::io::ErrorKind`                          |
| `Read`                  | A file, archive or entry cannot be read: refused, out of bounds, or undecodable |
| `Assertion`             | A binary-format check failed, from `xrf_utils`' `assert*` helpers               |
| `ChunkNotEnded`         | A chunk reader stopped with `remaining` bytes unread                            |
| `NoTerminator`          | A fixed or null-terminated string has no terminator                             |
| `Encoding`              | Text cannot be written in the required encoding, or a stream cannot be coded    |
| `Serialization`         | A value cannot be written to its binary format                                  |
| `LtxParse`, `LtxScheme` | LTX text fails to parse at `line:col`, or a scheme declaration is malformed     |
| `Convert`               | An LTX value or structure cannot be converted to what the reader expects        |
| `Verify`                | A verification run found problems                                               |
| `Asset`                 | An asset resolves to nothing or cannot be used                                  |
| `TextureProcessing`     | Image decoding or encoding failed                                               |
| `UnknownLanguage`       | A translation language name is not one XRF knows                                |
| `NotImplemented`        | The requested operation or format variant is not supported                      |
| `Cancelled`             | The operation stopped at a safe boundary because cancellation was requested     |
| `Unexpected`            | An internal state that should not occur                                         |
| `Serde`, `Generic`      | Converted from `serde_json` and boxed `dyn Error` failures                      |

`Cancelled` is control flow, not a failure: it lets a cancellation check compose with `?` and stop a parallel iterator.
An operation catches its own `Cancelled` and reports what it completed, so one reaching a caller means a missed catch.
See [xrf-job](../xrf-job/README.md) for cancellation.

## Conversions

`?` converts these errors through the `From` impls beside [the variants](src/xrf_error.rs):

| Source                         | Variant             |
| ------------------------------ | ------------------- |
| `std::io::Error`               | `Io`, with its kind |
| `serde_json::Error`            | `Serde`             |
| `image::ImageError`            | `TextureProcessing` |
| `Box<dyn Error + Send + Sync>` | `Generic`           |

Any other error needs an explicit `map_err` to the variant that describes it. Wrapping everything as `Invalid` loses
distinctions callers act on, such as a missing file (`Io` with `ErrorKind::NotFound`) against an unreadable one.

## Boundaries and checks

`XrfError` does not serialize. A boundary - an IPC command, a CLI report - carries one as its `Display` text, and what
a surface reports about the data it judged is that domain's own finding type (`xrf_ltx::LtxFinding`,
`xrf_environment::EnvironmentFinding`), never an error. A config that breaks its scheme is such a finding; `LtxScheme`
is a scheme file that cannot be read as a declaration. The crate does not log errors, choose exit codes, or decide how a
UI shows them.

Run `cargo test --locked -p xrf-error` from the repository root. See the [variants, constructors and conversions](src/xrf_error.rs)
and the [result alias](src/xrf_result.rs).
