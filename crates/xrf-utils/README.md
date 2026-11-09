# xrf-utils

Small primitives every XRF crate shares:

- bounds for counts read from files;
- safe host paths for names taken from data;
- staged file publication;
- line endings, text encodings and natural ordering;
- the workspace's units for durations and sizes.

It depends only on `xrf-error` among XRF crates, so any crate can use it. Domain rules live in the domain crates.

Everything is exported flat from the crate root (`xrf_utils::format_bytes`, `xrf_utils::HostDestination`). The sources
are grouped like the sections below: `binary/`, `host_path/`, `staging/`, `text/` and `units/`.

## Counts and sizes in binary formats

A count read from a file is untrusted: reserving memory for a hostile one aborts the process instead of failing the
parse.

- `assert_count_fits(count, budget, min_record_size, what)` refuses a count that cannot fit the remaining budget;
  `new_bounded_vec` also reserves it, fallibly. `budget` and `min_record_size` share a unit (bytes, for a binary
  reader). Understate the minimum: overstating it rejects valid input.
- `new_declared_vec` is for a length nothing bounds, such as a decompressed size.
- `narrow_format_size::<u8 | u16 | u32>(value, what)` narrows a length into its format field on write, where a bare `as`
  would silently truncate.
- `assert`, `assert_equal`, `assert_not_equal` and `assert_length` return `XrfError::Assertion` for format checks.
- `read_exact_at` reads at an offset without moving the file's cursor, so one handle can serve several threads.

```rust
use xrf_utils::{new_bounded_vec, narrow_format_size};

fn main() -> xrf_error::XrfResult {
  // Four records of at least 8 bytes fit in 32 remaining bytes; five do not.
  let records: Vec<u64> = new_bounded_vec(4, 32, 8, "test records")?;

  assert!(records.is_empty());
  assert!(new_bounded_vec::<u64>(5, 32, 8, "test records").is_err());

  let count: u16 = narrow_format_size(300, "test records")?;

  assert_eq!(count, 300);
  assert!(narrow_format_size::<u8>(300, "test records").is_err());

  Ok(())
}
```

## Host paths for names taken from data

A name read from file content (an archive entry, an XML sprite id, an LTX section) reaches the host filesystem only
through these types:

- `HostFileName::new(name, what)` is one path component.
- `HostRelativePath::new(name, what)` splits on either separator and drops empty components.
- Both keep the authored spelling and refuse `.`, `..` and any colon on every host: Linux reads `..\x` and `C:\x` as one
  ordinary name, which would escape on Windows. `what` names the value in the error.
- `HostDestination::new(root, policy)` writes below a root, one component at a time:
  - `create_directory`, `create_file`;
  - `prepare_file`, which creates the parents and returns the path;
  - `resolve_file`, which checks what exists and creates nothing, for a writer that reads its target first or has a dry
    run.
- `HostLinkPolicy::Follow` (the default) writes through links, junctions and mod-manager overlays. `Refuse` refuses any
  link below the root, and on Windows any reparse point, for an untrusted destination. The root itself is never
  inspected.

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

Display helpers are lossy and must never be used as addresses:

- `format_path` renders a host path for people; `format_path_or` names an absent one with the caller's word.
- `format_portable_path` gives a `/`-separated report key.
- `resolve_absolute_path` resolves against the current directory lexically, without touching the filesystem or following
  links.

Engine logical paths and mounted lookup belong to [xrf-vfs](../xrf-vfs/README.md); file extensions to
[xrf-extension](../xrf-extension/README.md).

## Publish a file

```rust,no_run
use std::path::Path;

use xrf_utils::StagedFile;

fn main() -> std::io::Result<()> {
  StagedFile::write_creating_directories(Path::new("output/report.txt"), b"Verification completed\n")
}
```

A staged write leaves the destination holding either the previous file or the new one:

1. It creates a hidden sibling (`.<name>.xrf-tmp-<pid>-<n>`), writes it, and calls `sync_all`.
2. It renames the sibling over the destination.
3. It syncs the parent directory, best effort.

- A replacement keeps the old file's permissions; a failed write removes the sibling.
- `StagedFile::write` requires the parent directory; `write_creating_directories` creates it.
- Neither is create-only: both replace an existing file, so the caller enforces any overwrite policy.
- Both return `std::io::Result`, so the caller can name the artifact and keep the `ErrorKind`.

### Several files together

- Stage each with `StagedFile::stage(target, sibling, contents)`, naming the sibling with
  `StagedFile::name_sibling(target, purpose)`.
- Once all are staged, call `publish` on each.
- A `StagedFile` dropped unpublished removes its sibling.
- `StagedFile::is_sibling` recognizes any `.<name>.xrf-…` name, so a directory walk can skip leftovers.

### Testing failures

The `staging-faults` feature exposes, for a dependent crate's tests only:

- `StagingFaults::fail_next_write()`: the next staged write on the current thread fails with `StorageFull`;
- `StagingFaults::fail_publish_after(n)`: a publish fails once `n` more have succeeded.

## Text

```rust
use xrf_utils::{LineEndings, natural_cmp};

let source: &str = "first\r\nsecond\r\n";

assert_eq!(LineEndings::detect(source.as_bytes()), Some(LineEndings::Crlf));
assert_eq!(LineEndings::normalize(source), "first\nsecond\n");
assert_eq!(LineEndings::Crlf.apply("first\n"), "first\r\n");

let mut names: [&str; 3] = ["item10", "item2", "item1"];

names.sort_by(|left, right| natural_cmp(left, right));
assert_eq!(names, ["item1", "item2", "item10"]);
```

### Line endings

- `LineEndings::detect` returns the more frequent of LF and CRLF (LF on a tie), or `None` with neither. It ignores bare
  CR.
- `LineEndings::normalize` turns CRLF and bare CR into LF.
- `apply` rewrites every break as the ending it is called on, and adds no final one.
- `LineEndings` parses exactly `lf` and `crlf`.

### Natural order

`natural_cmp` orders identifiers the way rustfmt does under `style_edition = "2024"`:

- digit runs compare as numbers (`a2` before `a10`);
- an equal number spelled longer sorts first (`a00` before `a0`);
- `_` sorts before every other character.

Its tests pin it against real rustfmt output.

### Encodings

The encoding always comes from the format, never from guessing. Game text is Windows-1251.

- `new_windows1251_encoder` and its siblings return `encoding_rs` encodings (UTF-8, Windows-1250, 1251, 1252).
- `decode_bytes_to_string` handles a BOM; `decode_bytes_to_string_without_bom_handling` does not;
  `encode_string_to_bytes` encodes. Each returns `InvalidData` rather than substituting characters.
- Windows-1251 helpers: `decode_w1251_bytes_to_string`, `encode_string_to_w1251_bytes`,
  `read_as_string_from_w1251_encoded`.
- `encode_string_into_w1251_bytes` and `read_as_string_from_w1251_encoded` keep an ASCII buffer as it is instead of
  copying it, which matters for documents of tens of megabytes.
- Base64 helpers use the URL-safe alphabet without padding.

### Parsing

- `read_engine_integer` reads an integer as the engine's `atoi` does: the longest leading integer, or zero, saturated to
  `i32`.
- `vector_from_string(_sep)(_sized)` parse separated lists.
- `format_camel_case` turns a snake or kebab name into a JavaScript identifier.

## Durations, sizes and time

- Runtime durations are `Duration`. A serialized one is whole `u64` milliseconds, through
  `#[serde(with = "xrf_utils::duration_ms")]`, `optional_duration_ms` (`null` when unmeasured) or `duration_ms_vec`.
  The `duration_wire_format` test scans the workspace and fails for a `Duration` or `Option<Duration>` field of a
  `Serialize` type without one of these in the six lines above it.
- Sizes are `u64` bytes. `BYTES_PER_KILOBYTE` and its siblings are binary (1024); `megabytes_to_bytes` saturates.
- `format_duration` renders `742 ms`, `3.2 s` or `1 m 12 s`; `format_bytes` renders `512 B`, `1.5 KB` or `1.17 GB`.
  Both match the UI's `@xrf/format`. `format_bytes_pair` renders two sizes for comparison in the larger one's unit.
- `wall_clock_millis` is the wall clock in milliseconds since the Unix epoch, or zero for a clock set before it.

## Checks

Run `cargo test --locked -p xrf-utils` from the repository root. Functions return `std::io::Result` or `XrfResult` as
their signatures say; callers add domain context.

Sources: [host paths](src/host_path/mod.rs), [capacity bounds](src/binary/capacity.rs),
[staged writes](src/staging/staged_file.rs), [line endings](src/text/line_endings.rs) and
[encodings](src/text/encoding.rs).
