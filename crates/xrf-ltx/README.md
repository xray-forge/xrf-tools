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

`read_from_str` lowers one document under standard rules; it does not load an include tree. Use
`read_document_from_str` to retain statements and spans, and the source-preserving variant when the authored lines must
remain available.

For a standalone file, `Ltx::read_from_file_standard` applies the standard include and inheritance rules, and
`read_from_file_with_dialect` resolves under a dialect you pass. Both read the host filesystem only; use a project for
configs in archives.

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

Paths inside a project are logical: lower-case engine paths with `\` separators, held as `XrayLogicalPath`. Convert a
config name relative to the project with `config_path`, then pass it to `read_resolution`. `path_of` turns a logical
path back into the file a person can open, or leaves it logical for an archived config.

Ways to open a project:

| Door                      | Reads through                                                                      |
| ------------------------- | ---------------------------------------------------------------------------------- |
| `open_at_path[_opt]`      | One directory mounted at the logical root.                                         |
| `open_at_roots_opt`       | An `XrayRoots` search plan, mounted for this project.                              |
| `open_at_search_opt`      | A search the caller already holds, narrowed to an optional prefix such as configs. |
| `open_lean_at_search_opt` | The same, but lists nothing up front: for resolving a known root only.             |

Opening lists every `.ltx` file, reads each one's includes in parallel, and records entry points (`ltx_file_entries`,
configs no other config includes). An opened-over search keeps the caller's mounts and cache: nothing is mounted again,
and parsed documents are retained according to that VFS snapshot's cache policy. `search()` returns the project's
search for callers that read further assets through it.

`read_resolution` keeps the resolved root only when `LtxProjectOptions::is_caching_resolutions` is on (off by default;
a resolved modded `system.ltx` is tens of megabytes), and `forget_root` drops one. `resolve_explained` records where
every field came from (`LtxFieldOrigin`) and is never cached; plain reads do not collect provenance. `system_ltx` reads
the project's `system.ltx`.

## Dialects and other configs

Standard LTX is the default. Supply [xrf-dltx](../xrf-dltx/README.md)'s dialect with `LtxProjectOptions::with_dialect`;
the project keeps that choice for every later read. Use `read_full_in_search` to resolve a config outside the project's
prefix, such as a level's, under the project's dialect. `document_source` gives readers such as
[xrf-ltx-inspect](../xrf-ltx-inspect/README.md) the project's documents through its cache and read counters.

## Format and verify

`Ltx::format_from_str` renders canonical text for one document. On a project, `check_format_all_files_opt` checks every
config, archived ones included, without writing; `format_all_files_opt` rewrites files in place and refuses the whole
run when any config is archived. The format options carry output and job policy.

`LtxProjectOptions::is_with_schemes_check` loads section schemes (`scheme.ltx` and `*.scheme.ltx`) while the project
opens; a section opts in with a `$scheme` field. `is_strict_check` adds checks, currently that `#include` names match
file names case-sensitively. Both are off by default. Schemes judge resolved values, because only resolution can say
whether inheritance supplied a required field.

`verify_entries` checks every entry point, and `verify_entries_opt` adds progress and output options. `verify_resolved`
checks one entry the caller has already resolved, with the same rules. A sweep continues past an unreadable config:
inspect the result's `errors`, and its `outcome`, which says whether a cancelled run stopped early.

Canonical formatting writes CRLF line endings and preserves spaces inside section brackets: `[ base ]` and `[base]` are
different section names, while parent names after `:` are trimmed. Files are decoded as strict Windows-1251, so bytes
outside it are an error rather than replaced.

## Values and records

- `read_engine_bool`, `read_engine_float`, `scan_engine_float` and `read_engine_floats` read a value as the engine's
  `CInifile` does, including what it makes of malformed text.
- `Condlist` parses the `{+info} value, otherwise` condition lists that logic fields are written in.
- `LtxImportExport`, `read_ltx_field` and `read_ltx_optional_field` bind a typed record to a section.

## Module map

The crate is grouped by the stage each part answers for, and a config passes through them in this order:

- `syntax`: what the characters and reserved names mean.
- `document`: one file as it was written: the scan, the statements, and the canonical rendering back to text.
- `source`: where a document comes from, so a dialect never learns whether configs sit on disk or in volumes.
- `dialect`: which rules turn a tree of documents into a resolved config. Standard LTX is here; the Monolith DLTX
  patch dialect is `xrf-dltx`, which implements the same trait from outside.
- `ltx`: the resolved config a dialect answers with, and what a caller can do with one.
- `project`: a whole tree of configs, and the verify and format passes over it.

`scheme` and `condlist` are the two value languages a field may be written in, read on demand rather than as a stage.
`binding` is how a typed record is read out of a config and written back into one. Every module is private; the
supported names are re-exported from the crate root.

## Errors and checks

Parsing, include resolution, inheritance, and scheme checks fail at different stages, all as `XrfError`. A document that
parses need not resolve under the selected dialect.

Run `cargo test --locked -p xrf-ltx` from the repository root.

See the [exports](src/lib.rs) and the [project API](src/project/ltx_project.rs).
