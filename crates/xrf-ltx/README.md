# xrf-ltx

Parses, resolves, verifies, and formats X-Ray LTX configs, including includes, inheritance, and section schemes.

## Choose the representation

- `LtxDocument` represents one file as written, with statements and spans for formatting or inspection.
- `Ltx` and `Section` expose resolved config values through methods such as `get_from` and `section`.
- `LtxProject` reads a tree through the VFS and resolves it using the selected `LtxDialect`.

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

`read_from_str` lowers one document under standard rules; it does not load an include tree. Use the document form to
retain statements and spans, and the source-preserving variant when the authored lines must remain available.

## Read a project

```rust,no_run
use xrf_ltx::LtxProject;

fn main() -> xrf_error::XrfResult {
  let project = LtxProject::open_at_path("gamedata/configs")?;
  let entry = project.config_path("system.ltx")?;
  let config = project.read_full(&entry)?;

  println!("{} resolved sections", config.len());

  let explained = project.resolve_explained(&entry)?;

  println!("{} explained sections", explained.ltx.len());

  Ok(())
}
```

Open a config root with `LtxProject::open_at_path`, or use `open_at_path_opt` to supply options. Convert a relative
config name with `config_path`, then pass the logical path to `read_full`. Use `resolve_explained` when field origins
are needed; normal reads do not collect that provenance.

## Dialects and scope

Standard LTX is the default. Supply [xrf-dltx](../xrf-dltx/README.md)'s dialect with
`LtxProjectOptions::with_dialect` for DLTX. `open_at_scope_opt` opens configs over an existing VFS scope, so projects
can share mounted sources.

For a standalone file, `Ltx::read_from_file_standard` names the standard include/inheritance rules explicitly;
`read_from_file_with_dialect` accepts a selected dialect. Project reads retain that choice across later operations.
Use `read_full_in_scope` for related configs outside the project's own prefix.

## Format and verify

`Ltx::format_from_str` renders canonical text for one document. Project `check_format_all_files_opt` checks without
writing; `format_all_files_opt` rewrites physical files. The format options carry output and job policy.

Enable `is_with_schemes_check` to load section schemes during project assembly. `is_strict_check` enables additional
checks, including include-name case sensitivity. Both options are off by default. Schemes judge resolved values;
the syntactic document alone cannot answer whether inheritance supplied a required field.

Call `verify_entries` for the project's entry points, or `verify_entries_opt` to supply progress and output options.
`verify_resolved` checks one entry's already-resolved `Ltx`. Inspect the returned errors and outcome: a whole-tree pass
can retain findings from failed entries while continuing through the rest.

Canonical formatting uses CRLF and preserves spaces inside section brackets: `[ base ]` and `[base]` are different
names. File I/O uses strict Windows-1251. Archived configs can be read and checked, but in-place formatting requires
writable loose files.

## Errors and checks

Parsing, include resolution, inheritance, and scheme checks can fail at different stages. Use the typed `XrfError`
and its source context rather than treating every failure as malformed text. A successfully parsed document need not
be a valid resolved tree under the selected dialect.

Run `cargo test --locked -p xrf-ltx` from the repository root.

See the [exports](src/lib.rs), [project API](src/project/ltx_project.rs), and
[xrf-ltx-inspect](../xrf-ltx-inspect/README.md) for viewer-oriented records.
