# xrf-export

Extracts `extern(...)` declarations from XRF TypeScript into a manifest and renders it as JSON, XML or HTML. In
xrf-engine, `extern("name", value)` (from `xray16/lib`) publishes a function or value under a global name that the
game's configs and engine callbacks reach; the declarations live under `src/engine/declarations`. The CLI command
`xrf-cli externs export` writes or checks the manifest artifact, and the desktop app's exports browser reads the same
declarations.

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

Parsing reads TypeScript source and resolves imported symbols through `tsconfig.json` path aliases
([xrf-typescript](../xrf-typescript/README.md)); it executes nothing and needs no game runtime.

## Source selection

`parse_directory` walks the declarations root and keeps `.ts` files (any case) whose text contains `extern` followed by
`(`. It skips `.test.ts`, `.spec.ts`, anything under a `__test__` directory, and the `.git`, `node_modules`, `target`
and `dist` directories. Files are parsed in sorted path order, and only a source's top-level `extern(...)` calls
declare anything. It is an extractor for this one declaration contract, not a general TypeScript documentation
generator, and a successful extraction does not prove the export is callable in the game.

## What the runtime does with a call

`xray16/lib`'s `extern(key, value)` splits the key on dots, makes a table of each missing segment under `_G`, and
assigns the value at the last one, replacing whatever was there and everything under it. So the parser refuses two
calls of which the game would keep at most one: in different modules, one at or inside the other's key, since the
order the modules load in decides; in one module, which runs its calls in source order, one written before an object
literal at a key around it, or one writing inside a value that is not an object literal. Calls that only share a root,
like every `xr_effects.*` effect, each add a name to the one table.

## Manifest and project views

`ParsedExternManifest` holds the canonical `manifest` and the `parsed` declarations with their source locations
(`ExternSourceLocation`: path, line, column and inclusive `end_line`). Source paths are relative to the declarations
root, with `/` separators.

`ExternManifest::exports` maps each exported name, sorted, to `ExternExport::Callable` (parameters, return type,
optional JSDoc description and `@returns`, per-parameter docs) or `ExternExport::Value` (its asserted type). A callable
parameter or return without a type annotation is recorded as `unknown`. A function passed by reference must have a
resolvable type; otherwise parsing fails with a message suggesting `name as (arg: Type) => ReturnType`.

`ExportsProjectParser::parse_project_from_path` builds the desktop app's `ExportsProject`: the same parse projected into
serializable `ExportDescriptor`s, plus the manifest so the app can publish the artifact without parsing again.
`ExportsProject::read_declaration_source(name)` returns a declaration's source lines; it reads only files of
declarations the project already parsed, and returns whatever lines remain if the file has shrunk since.

## Output formats

`ExternManifest::render` returns the rendered text with a final newline; it does not write a file. A `Some` line-ending
argument overrides the format's default.

| Format | Content                                          | Default endings |
| ------ | ------------------------------------------------ | --------------- |
| `Json` | The manifest, pretty-printed; the tracked format | CRLF            |
| `Xml`  | `<externs><exports>`, one `<export>` per name    | LF              |
| `Html` | A reference page grouped by namespace            | LF              |

`ExternManifest::write` renders and publishes through `xrf_utils::StagedFile::write_creating_directories`, creating
parent directories, so a failed write leaves the previous artifact whole. `ExternManifest::verify_artifact` checks an
existing artifact: a JSON one must read back as the same manifest, an XML or HTML one must match a fresh render whatever
its line endings. `ExternFormat::from_extension` infers the format from `.json`, `.xml`, `.html` or `.htm`, and
`ExternFormat`'s `FromStr` takes the `--format` names in `ExternFormat::NAMES`.

## Errors and checks

Failures split by cause, which `externs export --check` relies on to tell a finding from a run that could not happen:

- `Parsing` or `Invalid` for content the parser judged: TypeScript that does not parse, a declaration it cannot read a
  contract from, a name declared twice, or two calls writing over each other (`Invalid`, naming both sources).
- `Io` for a root that is not a directory, an unreadable source or artifact, and a failed publication.
- `Serde` when JSON rendering fails; an unsupported extension or format name, and a JSON artifact that does not read as
  a manifest, are `Invalid`; an artifact describing another manifest is `Verify`.

Run `cargo test --locked -p xrf-export` from the repository root. `manifest` holds the model, `parse` the source
selection and the per-call parsers under `parse::declaration`, `render` the formats and their renderers, and `project`
the desktop app's view. See the [parser](src/parse/extern_manifest_parser.rs), [manifest](src/manifest/extern_manifest.rs)
and [formats](src/render/extern_format.rs).
