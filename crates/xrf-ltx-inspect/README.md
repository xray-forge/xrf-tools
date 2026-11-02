# xrf-ltx-inspect

Turns an [xrf-ltx](../xrf-ltx/README.md) project into serializable records for config browsers: inventory, structure,
resolved fields, scheme reports, source lines, and anchored findings.

The records feed the desktop app's configs explorer and `xrf-cli ltx inspect`. Inspection sits on top of `xrf-ltx`
rather than inside it: the core is the shared reader every app and command resolves configs through, and nothing there
should grow a field because a viewer wants one. Every record is serialized camelCase, anchored to source lines, and
carries the `entry` point it was computed against, so a consumer can key a cache by it and never computes a line number
itself. The crate reads; it does not select dialects or edit files, and it never depends on `xrf-dltx`.

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

  let verification = project.verify_resolved(&entry, &resolution);
  let findings = reader.read_findings(&verification.findings)?;

  println!("{} findings", findings.len());

  Ok(())
}
```

The root reader borrows the resolution and document source. Keep both alive while requesting structure, sections, or
findings from that reader.

## Choose a scope

- `LtxInventoryReader` describes the whole project: every config, its source mount, whether a loose file backs it, which
  configs include it, and its `LtxInventoryRole`.
- `LtxRootReader::new` borrows an entry point, the name of the dialect that resolved it, its resolution, and a document
  source. Use `read_structure` for authored sections, `read_index` and `read_sections` for resolved values,
  `read_section_scheme` for scheme reports, and `read_findings` for diagnostics. It is one reader rather than one per
  question because every question needs the same entry point, resolution and source, and a surface showing a file
  beside its findings is one screen.
- `LtxTextReader` returns source lines without requiring a project or resolution.

Resolve with `LtxProject::resolve_explained` when field provenance is needed; a plain resolution reports origins as
`Unrecorded` rather than guessing. Use the project's `document_source` so archived and loose configs follow the same
read path. Readers take `&dyn LtxDocumentSource` and an `LtxResolution`, so they accept standard and DLTX results
alike, and tests can serve configs from a map of strings.

A config has one role, by precedence `SchemeFile`, `Attachment`, `EntryPoint`, `Included`: a scheme file that is also an
entry point reports `SchemeFile`. An `Attachment` (a DLTX patch file) names no root it patches.

## Authored structure and resolved values

`read_structure` describes a config as written in the context of one resolved root. Pass the entry points that reach the
config, which `LtxInventory::list_entry_points_of` answers. A file that will not parse is still answered, with its parse
error recorded, so a viewer can open it. Keys written before a config's first section header are reported in
`root_entries`.

`read_index` lists section names, parents, origins, and field counts in the dialect's section order, not sorted for
display. `read_sections` fetches named sections in the order asked, skipping names the root does not hold, so a browser
can request detail a page at a time.

Declared parents come from the document source because resolution flattens inheritance. One reader reads each declaring
config once however many questions it answers, so keep a reader for a root rather than building one per request.

Pass the project's `ltx_scheme_declarations` to `with_declared_schemes` before `read_section_scheme`. Without them the
reader reports every binding as undeclared rather than assuming a scheme exists because a config names it.

## Read source lines

```rust
use xrf_ltx_inspect::LtxTextReader;

fn main() {
  let text = LtxTextReader::read("actor.ltx", "[actor]\nhealth = 100\n");

  assert!(text.is_normalized);
  assert_eq!(text.lines.len(), 3);
}
```

The final newline produces a trailing empty line, matching an editor's line count, so the lines of an LF file joined
with `\n` reproduce it. `is_normalized = true` says the lines came through the parser, which loses one thing: a line of
only whitespace comes back empty. An empty file has no lines. Text that will not parse is still returned, split on
newlines, with `is_normalized = false` so a viewer can show what needs repair.

## Findings and errors

A finding is the core's `xrf_ltx::LtxFinding`; this crate adds only the line. `read_findings` places what verifying a
root found (`LtxEntryVerification::findings`, which already holds the dialect's resolution diagnostics) on source lines.
`read_file_findings` reports problems of one file from its structure: a parse error, an include that reached nothing, or
keys before the first header in a config that also declares sections (`RootKeys`). `LtxAnchoredFinding` serializes flat,
the finding's fields beside `line`. Storage and other read failures use `XrfResult`. Keep the entry point with cached
results because a shared config can resolve differently under another root.

## Module map

The record groups are named for the question each answers:

- `inventory`: what the project contains, and which role each config plays in it.
- `structure`: one config as written, minus what a syntax highlighter can already see.
- `resolved`: one resolution, indexed and sliceable a page of sections at a time.
- `scheme`: what judges one section, and how the section measures against it.
- `findings`: what is wrong, anchored to the line a person has to open.
- `text`: one config's lines, as the authored view renders them.

`ltx_root_reader.rs` holds `LtxRootReader`, which answers across the `structure`, `resolved`, `scheme` and `findings`
groups for one root.

## Checks

Run `cargo test --locked -p xrf-ltx-inspect`. The optional `typescript-bindings` feature derives `specta` types for the
frontend's generated bindings.

See the [root reader](src/ltx_root_reader.rs), [record exports](src/lib.rs), and [tests](src/tests).
