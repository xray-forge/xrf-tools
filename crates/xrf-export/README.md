# xrf-export

Extracts `extern(...)` declarations from XRF's TypeScript into a manifest, and renders it as JSON, XML or HTML.

In xrf-engine, `extern("name", value)` (from `xray16/lib`) publishes a function or value under a global name that game
configs and engine callbacks can reach. The declarations live under `src/engine/declarations`.

- `xrf-cli externs export` writes or checks the manifest file.
- The desktop app's exports browser reads the same declarations.

## Extract and render declarations

```rust,no_run
use std::path::Path;

use xrf_export::{ExternFormat, ExternManifestParser, ParsedExternManifest};

fn main() -> xrf_error::XrfResult {
  let parsed: ParsedExternManifest = ExternManifestParser::new().parse_directory(Path::new("src/engine/declarations"))?;
  let json: String = parsed.manifest.render(ExternFormat::Json, None)?;

  println!("{} exports", parsed.manifest.exports.len());
  print!("{json}");

  Ok(())
}
```

Parsing reads TypeScript and resolves imported symbols through `tsconfig.json` path aliases
([xrf-typescript](../xrf-typescript/README.md)). It runs nothing and needs no game.

A successful extraction does not prove an export is callable in the game.

## Which files are read

`parse_directory` walks the declarations root:

- It keeps `.ts` files (any case) whose text contains `extern` followed by `(`.
- It skips `.test.ts`, `.spec.ts`, anything under `__test__`, and the `.git`, `node_modules`, `target` and `dist`
  directories.
- Files are parsed in sorted path order.
- Only a file's top-level `extern(...)` calls declare anything.

## Conflicting declarations

At runtime, `extern(key, value)` splits the key on dots, creates a table under `_G` for each missing segment, and assigns
the value at the last one, replacing whatever was there and everything under it.

The parser therefore refuses two calls when the game would keep at most one of them:

- **In different modules**, when one key is at or inside the other: the module load order would decide.
- **In one module** (calls run in source order), when a call is followed by an object literal at a key around it, or
  writes inside a value that is not an object literal.

Calls that only share a root, like every `xr_effects.*` effect, each add a name to the same table and are fine.

## Manifest and project views

`ParsedExternManifest` holds:

- `manifest`, the canonical manifest;
- `parsed`, the declarations with source locations (`ExternSourceLocation`: path, line, column and inclusive
  `end_line`). Paths are relative to the declarations root, with `/` separators.

`ExternManifest::exports` maps each exported name, sorted, to one of:

- `ExternExport::Callable`: parameters, return type, optional JSDoc description and `@returns`, per-parameter docs.
- `ExternExport::Value`: its asserted type.

Typing rules:

- A callable parameter or return without a type annotation is recorded as `unknown`.
- A function passed by reference must have a resolvable type. Otherwise parsing fails, suggesting
  `name as (arg: Type) => ReturnType`.

For the desktop app:

- `ExportsProjectParser::parse_project_from_path` builds an `ExportsProject`: the same parse as serializable
  `ExportDescriptor`s, plus the manifest, so the app can write the file without parsing again.
- `ExportsProject::read_declaration_source(name)` returns a declaration's source lines. It reads only files the project
  already parsed, and returns whatever lines remain if a file has since shrunk.

## Output formats

`ExternManifest::render` returns the text with a final newline; it writes no file. A `Some` line-ending argument
overrides the format's default.

| Format | Content                                          | Default endings |
| ------ | ------------------------------------------------ | --------------- |
| `Json` | The manifest, pretty-printed; the tracked format | CRLF            |
| `Xml`  | `<externs><exports>`, one `<export>` per name    | LF              |
| `Html` | A reference page grouped by namespace            | LF              |

- `ExternManifest::write` renders and writes through `xrf_utils::StagedFile::write_creating_directories`, so a failed
  write leaves the previous file whole.
- `ExternManifest::verify_artifact` checks an existing file. A JSON file must read back as the same manifest; an XML or
  HTML file must match a fresh render, ignoring line endings.
- `ExternFormat::from_extension` infers the format from `.json`, `.xml`, `.html` or `.htm`. `ExternFormat`'s `FromStr`
  takes the `--format` names in `ExternFormat::NAMES`.

## Errors

Errors split by cause, so `externs export --check` can tell a finding from a run that could not complete:

| Variant              | Cause                                                                                                                                          |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `Parsing`, `Invalid` | TypeScript that does not parse, an unreadable contract, a duplicate name, or two calls overwriting each other (`Invalid`, naming both sources) |
| `Invalid`            | An unsupported extension or format name, or a JSON file that is not a manifest                                                                 |
| `Verify`             | A file describing a different manifest                                                                                                         |
| `Io`                 | A root that is not a directory, an unreadable source or file, a failed write                                                                   |
| `Serde`              | JSON rendering failed                                                                                                                          |

## Checks

Run `cargo test --locked -p xrf-export` from the repository root.

| Module     | Holds                                                                 |
| ---------- | --------------------------------------------------------------------- |
| `manifest` | The model                                                             |
| `parse`    | Source selection, and the per-call parsers under `parse::declaration` |
| `render`   | The formats and their renderers                                       |
| `project`  | The desktop app's view                                                |

Sources: the [parser](src/parse/extern_manifest_parser.rs), [manifest](src/manifest/extern_manifest.rs) and
[formats](src/render/extern_format.rs).
