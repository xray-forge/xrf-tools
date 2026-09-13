# xrf-utils

Shared filesystem, encoding, formatting, sorting, and size helpers used by XRF crates.

## Normalize text for output

```rust
use xrf_utils::{LineEndings, apply_line_endings, detect_line_endings, natural_cmp, normalize_line_endings};

fn main() {
    let source = "first\r\nsecond\r\n";
    assert_eq!(detect_line_endings(source.as_bytes()), Some(LineEndings::Crlf));

    assert_eq!(normalize_line_endings(source), "first\nsecond\n");
    assert_eq!(apply_line_endings("first\n", LineEndings::Crlf), "first\r\n");

    let mut names = ["item10", "item2", "item1"];

    names.sort_by(|left, right| natural_cmp(left, right));
    assert_eq!(names, ["item1", "item2", "item10"]);
}
```

`detect_line_endings` selects the dominant ending, prefers LF on a tie, and returns `None` when there is no newline.
Normalization also converts bare CR to LF. Applying endings rewrites the existing breaks; it does not add a final one.

## Choose a helper

- Use the encoding helpers for engine text and `LineEndings` helpers for detecting or applying newline styles.
- Use `natural_cmp` for natural identifier ordering and `format_bytes` / `format_duration` for display.
- Use `write_file_staged` to replace a file through a staged write, or `write_new_file_staged` to also create parent
  directories. Both can replace an existing target. The optional `staging-faults` feature supports publication-failure
  tests.

## Publish a file

```rust,no_run
use std::path::Path;
use xrf_utils::write_new_file_staged;

fn main() -> std::io::Result<()> {
  write_new_file_staged(Path::new("output/report.txt"), b"Verification completed\n")
}
```

Staged writes create a sibling file, write and sync its contents, then rename it into place. `write_file_staged`
requires existing parent directories; `write_new_file_staged` creates them. The latter's name does not mean create-only:
callers must enforce an overwrite policy themselves. A successful replacement is followed by a best-effort parent sync.

## Encodings and paths

The encoding factories provide UTF-8 and Windows-1250/1251/1252. Decoding helpers distinguish BOM-aware reading from
reading without BOM handling. Encoding and decoding failures are reported rather than silently replacing characters.
Choose the encoding from the file format or language contract; these helpers do not infer it from arbitrary content.

Display paths such as `format_path` and `to_portable_path_string` may be lossy; do not turn them back into write
addresses. Engine logical paths and mounted asset lookup belong to [xrf-vfs](../xrf-vfs/README.md), and reading or
comparing a file extension belongs to [xrf-extension](../xrf-extension/README.md) — X-Ray's filename rules and the
vocabulary they answer in sit together there, below both this crate and the VFS.

## Checks

Run `cargo test --locked -p xrf-utils`. Enable `staging-faults` only when a dependent test needs to exercise a failed
publication. Helpers return either `std::io::Result` or `XrfResult` according to their API; callers add domain context.

See the [exports](src/lib.rs), [encoding helpers](src/encoding_utils.rs),
and [staged writes](src/staged_write_utils.rs).
