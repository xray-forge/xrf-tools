# xrf-ltx

Parses, resolves and formats X-Ray LTX configs, including `#include`s, section inheritance, and section schemes.

LTX is the INI-like format of everything under `gamedata\configs`. A config is resolved from an entry point such as
`system.ltx`: includes are read in, and a section written `[child]:parent` copies the fields of its parents. This crate
owns that model for the standard (vanilla) rules; [xrf-dltx](../xrf-dltx/README.md) adds Monolith's DLTX patch dialect
through the same `LtxDialect` trait.

It is the format alone, with no VFS. A whole tree read through an installation's volumes, with its verify and format
runs, is [xrf-ltx-project](../xrf-ltx-project/README.md).

## Choose the representation

- `LtxDocument` is one file as written: statements with spans, for formatting or inspection. It performs no I/O and
  applies no dialect.
- `Ltx` and `Section` are a resolved config. Read it with `section`, `get_from`, `iter` and `len`; sections and fields
  keep their written order.
- `LtxResolution` is what resolving one root answers: the `Ltx`, field `provenance` when asked for, and the dialect's
  non-fatal `diagnostics`.
- `LtxDocumentSource` is where a dialect reads documents from. `xrf-ltx` answers it from the host filesystem;
  `xrf-ltx-project` answers it from a VFS search.

## Read one document

```rust
use xrf_ltx::Ltx;

fn main() -> xrf_error::XrfResult {
  let text = "[actor]\nhealth = 100\n";
  let config = Ltx::read_from_str(text)?;

  assert_eq!(config.get_from("actor", "health"), Some("100"));

  let document = Ltx::read_document_from_str_preserving_source(text)?;

  assert!(!document.get_source_lines().is_empty());

  Ok(())
}
```

- `read_from_str` resolves one document under standard rules; it does not load includes.
- `read_document_from_str` keeps statements and spans; the source-preserving variant also keeps the authored lines.
- For a file on disk, `Ltx::read_from_file_standard` applies the standard include and inheritance rules, and
  `read_from_file_with_dialect` resolves under a dialect you pass. Both read the host filesystem only; use
  [xrf-ltx-project](../xrf-ltx-project/README.md) for configs in archives.

## Dialects

- Standard LTX is `LtxStandardDialect`; [xrf-dltx](../xrf-dltx/README.md) adds DLTX. Either resolves a root through
  any `LtxDocumentSource` with `resolve`.
- Both dialects read lines as the game does. A comment starts at the first `;` or `//`, except inside the first pair of
  quotes on the line, so `x = "a;b"` keeps `"a;b"`. Section names ignore case: `[Weapon]` and `[weapon]` are the same
  section, so declaring both is an error, and `Ltx::section("WEAPON")` finds it.

## Format

- `Ltx::format_from_str` renders canonical text for one document.
- `Ltx::is_formatted` says whether bytes are already canonical.
- `Ltx::format_file` judges one file on disk and, when asked, rewrites it.

Canonical text:

- uses CRLF line endings;
- keeps spaces inside section brackets, since `[ base ]` and `[base]` are different sections; parent names after `:`
  are trimmed.

Files are decoded as strict Windows-1251: a byte outside it is an error, not replaced.

## Schemes

A section opts into a scheme with a `$scheme` field, and scheme files (`scheme.ltx` and `*.scheme.ltx`) declare them.
`LtxSchemeParser::parse_from_source` reads a set of scheme files through any `LtxDocumentSource`, refusing a
declaration two files repeat; `parse_from_ltx` reads one already-resolved file. Checking a tree against them is
[xrf-ltx-project](../xrf-ltx-project/README.md)'s verification.

## Values and records

- `read_engine_bool`, `read_engine_float`, `scan_engine_float` and `read_engine_floats` read a value as the engine's
  `CInifile` does, including what it makes of malformed text.
- `Condlist` parses the `{+info} value, otherwise` condition lists that logic fields are written in.
- `LtxImportExport` binds a typed record to a section:
  - It reads through `LtxRead` and `LtxSectionRead`; `read_field` and `read_optional_field` parse strictly.
  - It writes through `LtxWrite` and `LtxSectionWrite`, which `Ltx` implements.
- A file of records is read and written without a config:
  - `LtxRecordReader` hands over each section as it is parsed (`import_all`, `import_indexed`).
  - `LtxRecordWriter` appends each section's text.
  - Both give the same bytes and errors a config would.

## Module map

A config passes through these stages in order:

| Module     | Answers                                                                                      |
| ---------- | -------------------------------------------------------------------------------------------- |
| `syntax`   | What the characters and reserved names mean                                                  |
| `document` | One file as written: the scan, the statements, and canonical rendering                       |
| `source`   | Where a document comes from, so a dialect never knows whether it is on disk or in an archive |
| `dialect`  | Which rules turn documents into a resolved config; standard LTX here, DLTX in `xrf-dltx`     |
| `ltx`      | The resolved config and what a caller can do with it                                         |

Outside the stages:

- `scheme` and `condlist` are the two value languages a field may be written in, read on demand.
- `binding` reads and writes typed records.

Every module is private; the supported names are re-exported from the crate root.

## Errors

- Parsing, include resolution and inheritance fail at different stages, all as `XrfError`.
- A scheme file that cannot be read as a declaration is `LtxScheme`.
- A config that breaks a scheme is a finding, not an error.
- A document that parses need not resolve under the selected dialect.

## Checks

Run `cargo test --locked -p xrf-ltx` from the repository root.

Sources: the [exports](src/lib.rs) and the [resolved config](src/ltx/ltx.rs).
