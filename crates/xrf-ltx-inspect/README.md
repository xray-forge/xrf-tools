# xrf-ltx-inspect

Turns an [xrf-ltx](../xrf-ltx/README.md) project into serializable records for config browsers: inventory, structure,
resolved fields, scheme reports, source lines, and findings anchored to lines.

The records feed the desktop app's configs explorer and `xrf-cli ltx list`, `find` and `inspect`.

- Every record serializes in camelCase, is anchored to source lines, and carries the `entry` point it was computed
  against. A consumer can key a cache by it and never computes a line number itself.
- The crate only reads. It does not select dialects or edit files, and it does not depend on `xrf-dltx`.
- It sits on top of `xrf-ltx` so that viewer needs never shape the core reader every command resolves configs through.

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

The root reader borrows the resolution and the document source; keep both alive while using it.

## Choose a reader

| Reader                 | Answers                                                                                                  |
| ---------------------- | -------------------------------------------------------------------------------------------------------- |
| `LtxInventoryReader`   | The whole project: every config, its mount, whether a loose file backs it, who includes it, and its role |
| `LtxRootReader`        | One resolved entry point: authored structure, resolved values, scheme reports, findings                  |
| `LtxDeclarationReader` | Where a section or field is written, across every config                                                 |
| `LtxTextReader`        | Source lines, with no project or resolution needed                                                       |

General rules:

- Resolve with `LtxProject::resolve_explained` when you need field origins. A plain resolution reports them as
  `Unrecorded` rather than guessing.
- Use the project's `document_source`, so archived and loose configs read the same way.
- Readers take `&dyn LtxDocumentSource` and an `LtxResolution`, so they accept standard and DLTX results alike, and
  tests can serve configs from a map of strings.

### Roles

Each config has one `LtxInventoryRole`, by precedence:

1. `SchemeFile`
2. `Attachment` (a DLTX patch file; it names no root it patches)
3. `EntryPoint`
4. `Included`

So a scheme file that is also an entry point reports `SchemeFile`.

## Structure and resolved values

`LtxRootReader::new` takes an entry point, the name of the dialect that resolved it, its resolution, and a document
source. One reader serves every question about that root, and reads each declaring config only once, so keep a reader
per root rather than building one per request.

| Method                | Returns                                                                          |
| --------------------- | -------------------------------------------------------------------------------- |
| `read_structure`      | A config as written, in the context of this root                                 |
| `read_index`          | Section names, parents, origins and field counts, in the dialect's section order |
| `read_sections`       | Named sections in the order asked, skipping names the root lacks, for paging     |
| `read_section_scheme` | How a section measures against its scheme                                        |
| `read_findings`       | Verification findings placed on source lines                                     |

Notes:

- `read_structure` takes the entry points that reach the config (`LtxInventory::list_entry_points_of`). A file that
  does not parse is still answered, with its parse error, so a viewer can open it. Keys before the first section header
  are in `root_entries`.
- Declared parents come from the documents, because resolution flattens inheritance.
- Pass the project's `ltx_scheme_declarations` to `with_declared_schemes` before `read_section_scheme`. Without them,
  every binding is reported as undeclared rather than assumed to exist.

## Find where something is written

`LtxDeclarationReader::new` takes an `LtxInventory` and a document source. `search` takes an `LtxDeclarationQuery` of
an optional section, field and value.

Matching follows resolution, so a found section is the one `read_sections` resolves under that name:

- Field names match whole and case-sensitively.
- Section names match whole and case-sensitively, unless `is_section_case_folded` is set (from
  `LtxDialect::is_folding_section_names`, which is true for DLTX).
- Values match as case-sensitive substrings.
- A query naming only a section answers headers; one naming a field or value answers field lines.

Each `LtxDeclaration` is authored text:

- It keeps the operation prefixes as written, on the statement and its header; `is_patch` marks either.
- One field can be declared in several places; only a resolution says which wins.
- A config that does not parse goes into `unreadable`, and the search continues.

`LtxDeclarationReader::list_holding_entry_points` answers the entry points whose resolution can hold a query's section,
so a caller resolves a few roots instead of all of them.

- It includes roots above each config with a header naming the section, or naming it as a parent (DLTX creates an empty
  section for an undeclared parent). The result is a superset.
- It answers `None` when an attachment could add the section (a plain or `@` header for it, or one naming it as a
  parent), because the dialect does not record which root an attachment patches.

## Read source lines

```rust
use xrf_ltx_inspect::LtxTextReader;

fn main() {
  let text = LtxTextReader::read("actor.ltx", "[actor]\nhealth = 100\n");

  assert!(text.is_normalized);
  assert_eq!(text.lines.len(), 3);
}
```

- The final newline produces a trailing empty line, matching an editor's line count; joining an LF file's lines with
  `\n` reproduces it.
- `is_normalized = true` means the lines came through the parser, which turns a whitespace-only line into an empty one.
- Text that does not parse is still returned, split on newlines, with `is_normalized = false`.
- An empty file has no lines.

## Findings and errors

A finding is the core's `xrf_ltx::LtxFinding`; this crate only adds the line.

- `read_findings` places what verifying a root found (`LtxEntryVerification::findings`, which already include the
  dialect's resolution diagnostics) on source lines.
- `read_file_findings` reports one file's own problems: a parse error, an include that reached nothing, or keys before
  the first header in a config that also has sections (`RootKeys`).
- `LtxAnchoredFinding` serializes flat: the finding's fields beside `line`.
- Storage and other read failures return `XrfResult`.

Keep the entry point with any cached result: a shared config can resolve differently under another root.

## Module map

| Module        | Answers                                                      |
| ------------- | ------------------------------------------------------------ |
| `inventory`   | What the project contains, and each config's role            |
| `declaration` | Where a section or field is written                          |
| `structure`   | One config as written, beyond what a syntax highlighter sees |
| `resolved`    | One resolution, indexed and readable a page at a time        |
| `scheme`      | What judges a section, and how it measures up                |
| `findings`    | What is wrong, on the line a person has to open              |
| `text`        | One config's lines, as the authored view shows them          |

`ltx_root_reader.rs` holds `LtxRootReader`, which answers across `structure`, `resolved`, `scheme` and `findings` for
one root.

## Checks

Run `cargo test --locked -p xrf-ltx-inspect`. The optional `typescript-bindings` feature derives `specta` types for the
frontend's generated bindings.

Sources: the [root reader](src/ltx_root_reader.rs), [record exports](src/lib.rs), and [tests](src/tests).
