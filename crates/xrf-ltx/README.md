# xrf-ltx

Parses, resolves, verifies, and formats X-Ray LTX configs, including `#include`s, section inheritance, and section
schemes.

LTX is the INI-like format of everything under `gamedata\configs`. A config is resolved from an entry point such as
`system.ltx`: includes are read in, and a section written `[child]:parent` copies the fields of its parents. This crate
owns that model for the standard (vanilla) rules; [xrf-dltx](../xrf-dltx/README.md) adds Monolith's DLTX patch dialect
through the same `LtxDialect` trait.

## Choose the representation

- `LtxDocument` is one file as written: statements with spans, for formatting or inspection. It performs no I/O and
  applies no dialect.
- `Ltx` and `Section` are a resolved config. Read it with `section`, `get_from`, `iter` and `len`; sections and fields
  keep their written order.
- `LtxResolution` is what resolving one root answers: the `Ltx`, field `provenance` when asked for, and the dialect's
  non-fatal `diagnostics`.
- `LtxProject` is a whole config tree read through an `xrf_vfs::XraySearch`, so loose and archived configs are read the
  same way, and resolved under the selected `LtxDialect`.

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
  `read_from_file_with_dialect` resolves under a dialect you pass. Both read the host filesystem only; use a project
  for configs in archives.

## Read a project

```rust,no_run
use xrf_ltx::LtxProject;

fn main() -> xrf_error::XrfResult {
  let project = LtxProject::open_at_path("gamedata/configs")?;
  let entry = project.config_path("system.ltx")?;
  let resolution = project.read_resolution(&entry)?;

  println!("{} resolved sections", resolution.ltx.len());

  let explained = project.resolve_explained(&entry)?;

  println!("{} sections carry field origins", explained.provenance.len());

  Ok(())
}
```

Paths inside a project are logical: lower-case engine paths with `\` separators, held as `XrayLogicalPath`.

- `config_path` converts a config name relative to the project; pass the result to `read_resolution`.
- `path_of` turns a logical path back into a file a person can open, or leaves it logical for an archived config.

### Open a project

| Door                      | Reads through                                                                      |
| ------------------------- | ---------------------------------------------------------------------------------- |
| `open_at_path[_opt]`      | One directory mounted at the logical root.                                         |
| `open_at_roots_opt`       | An `XrayRoots` search plan, mounted for this project.                              |
| `open_at_search_opt`      | A search the caller already holds, narrowed to an optional prefix such as configs. |
| `open_lean_at_search_opt` | The same, but lists nothing up front: for resolving a known root only.             |

Opening lists every `.ltx` file, reads each one's includes in parallel, and records the entry points
(`ltx_file_entries`: configs no other config includes).

- A project opened over a search keeps the caller's mounts and cache. Nothing is mounted again, and parsed documents
  are kept according to that VFS snapshot's cache policy.
- `search()` returns the project's search, for reading further assets through it.

### Resolve

- `read_resolution` resolves one root. It keeps the result only when `LtxProjectOptions::is_caching_resolutions` is on
  (off by default: a resolved modded `system.ltx` is tens of megabytes). `forget_root` drops a cached one.
- `resolve_explained` also records where every field came from (`LtxFieldOrigin`). It is never cached; plain reads
  collect no provenance.
- `system_ltx` reads the project's `system.ltx`.

## Dialects and other configs

- Standard LTX is the default. Pass [xrf-dltx](../xrf-dltx/README.md)'s dialect with
  `LtxProjectOptions::with_dialect`; the project keeps it for every later read.
- `read_full_in_search` resolves a config outside the project's prefix, such as a level's, under the project's dialect.
- `document_source` gives readers such as [xrf-ltx-inspect](../xrf-ltx-inspect/README.md) the project's documents
  through its cache and read counters.

## Format

`Ltx::format_from_str` renders canonical text for one document. On a project:

- `check_format_all_files_opt` checks every config, archived ones included, without writing.
- `format_all_files_opt` rewrites files in place, and refuses the whole run when any config is archived.

The options carry output and job settings.

Canonical text:

- uses CRLF line endings;
- keeps spaces inside section brackets, since `[ base ]` and `[base]` are different sections; parent names after `:`
  are trimmed.

Files are decoded as strict Windows-1251: a byte outside it is an error, not replaced.

## Verify

Options, both off by default:

- `LtxProjectOptions::is_with_schemes_check` loads section schemes (`scheme.ltx` and `*.scheme.ltx`) when the project
  opens. A section opts in with a `$scheme` field. Schemes judge resolved values, since only resolution knows whether
  inheritance supplied a required field.
- `is_strict_check` adds stricter checks; currently, that `#include` names match file names case-sensitively.

Calls:

- `verify_entries` checks every entry point; `verify_entries_opt` adds progress and output options.
- `verify_resolved` checks one entry the caller has already resolved, with the same rules.

Both answer `findings`, one `LtxFinding` for each:

- section or field that breaks its scheme;
- statement the dialect reports the engine ignores;
- entry point that does not resolve (the sweep continues past it).

Every finding fails the verification. `outcome` says whether a cancelled run stopped early. Findings name configs by
engine path; `with_reported_paths` renders them as files on disk where they are loose.

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
| `project`  | A whole config tree, with its verify and format passes                                       |

Outside the stages:

- `scheme` and `condlist` are the two value languages a field may be written in, read on demand.
- `binding` reads and writes typed records.
- `finding` is the one finding type the verifier, the explorer, the gamedata check and the CLI share.

Every module is private; the supported names are re-exported from the crate root.

## Errors

- Parsing, include resolution and inheritance fail at different stages, all as `XrfError`.
- A scheme file that cannot be read as a declaration is `LtxScheme`.
- A config that breaks a scheme is a finding, not an error.
- A document that parses need not resolve under the selected dialect.

## Checks

Run `cargo test --locked -p xrf-ltx` from the repository root.

Sources: the [exports](src/lib.rs) and the [project API](src/project/ltx_project.rs).
