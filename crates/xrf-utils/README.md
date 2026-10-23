# xrf-utils

Small primitives every XRF crate shares: bounds for counts read from files, safe host paths for names taken from data,
staged file publication, line endings, text encodings, natural ordering, and the workspace's units for durations and
sizes. It depends only on `xrf-error` among XRF crates, so any crate can use it; domain rules live in the domain crates.

Everything is exported flat from the crate root (`xrf_utils::format_bytes`, `xrf_utils::HostDestination`).

## Counts and sizes in binary formats

A count read from a file is untrusted: reserving from a hostile one aborts the process instead of failing the parse.

- `assert_count_fits(count, budget, min_record_size, what)` refuses a count that cannot fit the remaining budget, and
  `new_bounded_vec` also reserves it, fallibly. `budget` and `min_record_size` share a unit (bytes for a binary reader);
  understate the minimum, since overstating it rejects valid input. `new_declared_vec` is for a length nothing bounds,
  such as a decompressed size.
- `to_format_size::<u8 | u16 | u32>(value, what)` narrows a length into its format field on write, where a bare `as`
  would truncate silently.
- `assert`, `assert_equal`, `assert_not_equal` and `assert_length` return `XrfError::Assertion` for format checks.
- `read_exact_at` reads at an offset without moving the file's cursor, so one handle can serve several threads.

```rust
use xrf_utils::{new_bounded_vec, to_format_size};

fn main() -> xrf_error::XrfResult {
  // Four records of at least 8 bytes fit in 32 remaining bytes; five do not.
  let records: Vec<u64> = new_bounded_vec(4, 32, 8, "test records")?;

  assert!(records.is_empty());
  assert!(new_bounded_vec::<u64>(5, 32, 8, "test records").is_err());

  let count: u16 = to_format_size(300, "test records")?;

  assert_eq!(count, 300);
  assert!(to_format_size::<u8>(300, "test records").is_err());

  Ok(())
}
```

## Host paths for names taken from data

A name read from file content (an archive entry, an XML sprite id, an LTX section) reaches the host filesystem only
through these types:

- `HostFileName::new(name, what)` is one component. `HostRelativePath::new(name, what)` splits on either separator and
  drops empty components. Both keep the authored spelling and refuse `.`, `..` and any colon on every host, because
  Linux reads `..\x` and `C:\x` as one ordinary name that escapes on Windows. `what` names the value in the error.
- `HostDestination::new(root, policy)` writes below a root one component at a time: `create_directory`,
  `create_file`, `prepare_file` (creates the parents and returns the path), and `resolve_file` (checks what exists and
  creates nothing, for a writer that reads its target first or has a dry run).
- `HostLinkPolicy::Follow` (the default) writes through links, junctions and mod-manager overlays. `Refuse` refuses any
  link below the root, and on Windows any reparse point, for a destination that is not trusted. The root itself is
  never inspected.

```rust
use std::path::Path;

use xrf_utils::{HostFileName, HostRelativePath};

fn main() -> xrf_error::XrfResult {
  let sheet: HostRelativePath = HostRelativePath::new("ui\\ui_icons", "Sprite sheet")?;
  let sprite: HostFileName = HostFileName::new("ui_icon_PDA", "Sprite")?;

  assert_eq!(
    sheet.join(&sprite.with_extension("dds")).as_path(),
    Path::new("ui").join("ui_icons").join("ui_icon_PDA.dds")
  );
  assert!(HostRelativePath::new("..\\outside", "Entry").is_err());
  assert!(HostFileName::new("C:outside", "Sprite").is_err());

  Ok(())
}
```

Display forms are lossy and never write addresses: `format_path` renders a host path for people (`format_path_or`
names an absent one with the caller's word), and `to_portable_path_string` gives a `/`-separated report key.
`to_absolute_path` resolves against the current directory lexically, without touching the filesystem or following
links. Engine logical paths and mounted lookup belong to [xrf-vfs](../xrf-vfs/README.md); reading or comparing a file
extension belongs to [xrf-extension](../xrf-extension/README.md).

## Publish a file

```rust,no_run
use std::path::Path;

use xrf_utils::write_new_file_staged;

fn main() -> std::io::Result<()> {
  write_new_file_staged(Path::new("output/report.txt"), b"Verification completed\n")
}
```

A staged write creates a hidden sibling (`.<name>.xrf-tmp-<pid>-<n>`), writes and `sync_all`s it, renames it over the
destination, then best-effort syncs the parent directory, so the destination holds either the previous file or the new
one. A replacement keeps the old file's permissions; a failed write removes the sibling. `write_file_staged` requires
the parent directory; `write_new_file_staged` creates it. Neither is create-only: both replace an existing file, so the
caller enforces any overwrite policy. Both return `std::io::Result` so the caller can name the artifact and keep the
`ErrorKind`.

The `staging-faults` feature exposes `staging_faults::fail_next_staged_write()`, which makes the next staged write on
the current thread fail with `StorageFull`; enable it only in a dependent crate's tests. `open_export_file` is an
unstaged, truncating create.

## Text

```rust
use xrf_utils::{LineEndings, apply_line_endings, detect_line_endings, natural_cmp, normalize_line_endings};

let source: &str = "first\r\nsecond\r\n";

assert_eq!(detect_line_endings(source.as_bytes()), Some(LineEndings::Crlf));
assert_eq!(normalize_line_endings(source), "first\nsecond\n");
assert_eq!(apply_line_endings("first\n", LineEndings::Crlf), "first\r\n");

let mut names: [&str; 3] = ["item10", "item2", "item1"];

names.sort_by(|left, right| natural_cmp(left, right));
assert_eq!(names, ["item1", "item2", "item10"]);
```

- Line endings: `detect_line_endings` returns the more frequent of LF and CRLF, LF on a tie, and `None` with neither;
  it ignores bare CR. `normalize_line_endings` turns CRLF and bare CR into LF. `apply_line_endings` rewrites every
  break and adds no final one. `LineEndings` parses exactly `lf` and `crlf`.
- `natural_cmp` orders identifiers the way rustfmt does under `style_edition = "2024"`: digit runs compare as numbers
  (`a2` before `a10`), an equal number spelled longer sorts first (`a00` before `a0`), and `_` sorts before every
  other character. Its tests pin it against real rustfmt output.
- Encodings: `new_windows1251_encoder` and its siblings return `encoding_rs` encodings (UTF-8, Windows-1250/1251/1252).
  `decode_bytes_to_string` handles a BOM, `decode_bytes_to_string_without_bom_handling` does not, and
  `encode_string_to_bytes` encodes; each returns `InvalidData` rather than substituting characters. The encoding comes
  from the format, never from guessing; game text is Windows-1251 (`encode_string_to_w1251_bytes`). Base64 helpers use
  the URL-safe alphabet without padding.
- `read_engine_integer` reads an integer as the engine's `atoi` does: the longest leading integer, or zero, saturated
  to `i32`. `vector_from_string(_sep)(_sized)` parse separated lists, `to_camel_case` turns a snake or kebab name into a
  JavaScript identifier.

## Durations, sizes and time

- Runtime durations are `Duration`. A serialized one is whole `u64` milliseconds through
  `#[serde(with = "xrf_utils::duration_ms")]`, `optional_duration_ms` (`null` when unmeasured) or `duration_ms_vec`.
  The `duration_wire_format` test scans the workspace sources and fails for a `Duration` or `Option<Duration>` field of
  a `Serialize` type without one in the six lines above it.
- Sizes are `u64` bytes. `BYTES_PER_KILOBYTE` and its siblings are binary (1024); `megabytes_to_bytes` saturates.
- `format_duration` renders `742 ms`, `3.2 s` or `1 m 12 s`, and `format_bytes` renders `512 B`, `1.5 KB` or
  `1.17 GB`; both match the UI's `@xrf/format` output. `format_bytes_pair` renders two sizes a reader compares in the
  larger one's unit.
- `wall_clock_millis` is the wall clock in milliseconds since the Unix epoch, zero for a clock set before it.

## Checks

Run `cargo test --locked -p xrf-utils` from the repository root. Functions return `std::io::Result` or `XrfResult` as
their signatures say; callers add domain context. See [host paths](src/host_path/mod.rs),
[capacity bounds](src/capacity_utils.rs), [staged writes](src/staged_write_utils.rs),
[line endings](src/line_ending_utils.rs) and [encodings](src/encoding_utils.rs).
