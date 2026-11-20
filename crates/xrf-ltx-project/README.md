# xrf-ltx-project

A whole tree of X-Ray LTX configs, read through the VFS: an `LtxProject`, and the verify and format runs that walk it.

[xrf-ltx](../xrf-ltx/README.md) owns the format: documents, dialects and the resolved `Ltx`. This crate adds what a
tree needs on top. It reads loose and archived configs alike through an `xrf_vfs::XraySearch`, resolves each entry
point under the selected `LtxDialect`, and checks the tree against its section schemes. It also owns `LtxFinding`, the
finding type the verifier, the explorer, the gamedata check and the CLI share.

## Read a project

```rust,no_run
use xrf_ltx_project::LtxProject;

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
- `read_full_in_search` resolves a config outside the project's prefix, such as a level's, under the project's dialect.
- `document_source` gives readers such as [xrf-ltx-inspect](../xrf-ltx-inspect/README.md) the project's documents
  through its cache and read counters.

Standard LTX is the default dialect. Pass [xrf-dltx](../xrf-dltx/README.md)'s with `LtxProjectOptions::with_dialect`;
the project keeps it for every later read.

## Format

- `check_format_all_files_opt` checks every config, archived ones included, without writing.
- `format_all_files_opt` rewrites files in place, and refuses the whole run when any config is archived.
- `LtxFilesFormatter` does the same for a list of files on disk.

The options carry the job that reports progress and takes cancellation. Canonical text is the format crate's
`Ltx::format_from_str`.

## Verify

`LtxProjectOptions`, both off by default:

- `is_with_schemes_check` loads section schemes (`scheme.ltx` and `*.scheme.ltx`) when the project opens. A section
  opts in with a `$scheme` field. Schemes judge resolved values, since only resolution knows whether inheritance
  supplied a required field.
- `is_strict_check` requires `#include` names to match file names case-sensitively.

Calls:

- `verify_entries` checks every entry point; `verify_entries_opt` takes the job and an optional `listener`, which
  hears an `LtxVerifyEvent::FieldChecked` for each field checked against a scheme.
- `verify_resolved` checks one entry the caller has already resolved, with the same rules.

Both answer `findings`, one `LtxFinding` for each:

- section or field that breaks its scheme;
- statement the dialect reports the engine ignores;
- entry point that does not resolve (the sweep continues past it).

Every finding fails the verification. `outcome` says whether a cancelled run stopped early. Findings name configs by
engine path; `with_reported_paths` renders them as files on disk where they are loose.

## Print a run

Verify and format runs print nothing; they answer results. `LtxRunPrinter` prints the same lines for every surface
that runs one:

- `print_verify_start` and `print_verify_result` go before and after a verification; `new_listener` prints each
  checked field at verbose level, and answers none otherwise, so a quiet run builds no events.
- `print_format_start` and `print_format_result` go before and after a formatting run.

## Checks

Run `cargo test --locked -p xrf-ltx-project` from the repository root.

Sources: the [exports](src/lib.rs) and the [project API](src/project/ltx_project.rs).
