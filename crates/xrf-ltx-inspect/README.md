# xrf-ltx-inspect

Turns an [xrf-ltx](../xrf-ltx/README.md) project into serializable records for config browsers: inventory, structure,
resolved fields, scheme reports, source lines, and anchored findings.

## Inspect a resolved root

```rust,no_run
use xrf_ltx::LtxProject;
use xrf_ltx_inspect::{LtxInventoryReader, LtxRootReader};

fn main() -> xrf_error::XrfResult {
  let project = LtxProject::open_at_path("gamedata/configs")?;
  let source = project.document_source();
  let inventory = LtxInventoryReader::new(&project, &source).read()?;

  println!("{} config files", inventory.files.len());

  let entry = project.config_path("system.ltx")?;
  let resolution = project.resolve_explained(&entry)?;
  let reader = LtxRootReader::new(entry.as_str(), project.get_dialect().get_name(), &resolution, &source);
  let index = reader.read_index()?;

  println!("{} sections under {}", index.sections.len(), index.entry);

  let sections = reader.read_sections(&["actor"])?;

  println!("{} requested sections returned", sections.len());

  Ok(())
}
```

The root reader borrows the resolution and document source. Keep both alive while requesting structure, sections, or
findings from that reader.

## Choose a scope

- `LtxInventoryReader` describes the whole project and each config's role.
- `LtxRootReader::new` borrows an entry point, its resolution, and its document source. Use `read_structure` for
  authored sections, `read_index` and `read_sections` for resolved values, and `read_findings` for diagnostics.
- `LtxTextReader` returns source lines without requiring a project or resolution.

Resolve with `LtxProject::resolve_explained` when field provenance is needed, and use the project's
`document_source` so archived and loose configs follow the same read path. The readers accept either standard or
DLTX results; this crate does not select dialects or edit files.

## Authored structure and resolved values

`read_structure` describes a config as written in the context of one resolved root. The caller supplies the entry points
that reach that config. `read_index` lists section names, parents, origins, and field counts; `read_sections` fetches
selected sections, allowing a browser to request detail in batches.

Declared parents come from the document source because resolution flattens inheritance. The resolved index preserves
the dialect's section order rather than sorting it for display.

Use `with_declared_schemes` before `read_section_scheme` when scheme declarations are available. Without declarations,
the reader cannot label a binding as declared merely because a config mentions its name.

## Read source lines

```rust
use xrf_ltx_inspect::LtxTextReader;

fn main() {
  let text = LtxTextReader::read("actor.ltx", "[actor]\nhealth = 100\n");

  assert!(text.is_normalized);
  assert_eq!(text.lines.len(), 3);
}
```

The final newline produces a trailing empty line, matching an editor's line count. An empty file has no lines.
Unparseable text is still returned with `is_normalized = false` so a viewer can show what needs repair.

## Findings, errors, and checks

`read_findings` anchors supplied verification errors to source. `read_file_findings` reports structure-local problems.
Some parse failures are returned as inspectable records; storage and other read failures still use `XrfResult`.
Keep the entry point with cached results because a shared config can resolve differently under another root.

Run `cargo test --locked -p xrf-ltx-inspect`. The optional `typescript-bindings` feature adds frontend type metadata.

See the [root reader](src/ltx_root_reader.rs), [record exports](src/lib.rs), and [tests](src/tests).
