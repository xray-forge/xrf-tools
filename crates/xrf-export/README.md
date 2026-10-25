# xrf-export

Extracts `extern(...)` declarations from XRF TypeScript into a manifest and renders it as JSON, XML or HTML. In
xrf-engine, `extern("name", value)` (from `xray16/lib`) publishes a function or value under a global name that the
game's configs and engine callbacks reach; the declarations live under `src/engine/declarations`. The CLI command
`xrf-cli externs export` writes or checks the manifest artifact, and the desktop app's exports browser reads the same
declarations.

## Extract and render declarations

```rust,no_run
use std::path::Path;

use xrf_export::{ExternFormat, ExternManifestParser, ParsedExternManifest, render_extern_manifest};

fn main() -> xrf_error::XrfResult {
  let parsed: ParsedExternManifest = ExternManifestParser::new().parse_directory(Path::new("src/engine/declarations"))?;
  let json: String = render_extern_manifest(&parsed.manifest, ExternFormat::Json, None)?;

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
and `dist` directories. Files are parsed in sorted path order. It is an extractor for this one declaration contract,
not a general TypeScript documentation generator, and a successful extraction does not prove the export is callable in
the game.

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

`render_extern_manifest` returns the rendered text with a final newline; it does not write a file. A `Some` line-ending
argument overrides the format's default.

| Format | Content                                          | Default endings |
| ------ | ------------------------------------------------ | --------------- |
| `Json` | The manifest, pretty-printed; the tracked format | CRLF            |
| `Xml`  | `<externs><exports>`, one `<export>` per name    | LF              |
| `Html` | A reference page grouped by namespace            | LF              |

`write_extern_manifest` renders and publishes through `xrf_utils::StagedFile::write_creating_directories`, creating parent directories,
so a failed write leaves the previous artifact whole. `ExternFormat::from_extension` infers the format from `.json`,
`.xml`, `.html` or `.htm`, and `ExternFormat`'s `FromStr` takes the `--format` names in `ExternFormat::NAMES`.

## Errors and checks

Failures split by cause, which `externs export --check` relies on to tell a finding from a run that could not happen:

- `Parsing` or `Invalid` for content the parser judged: TypeScript that does not parse, a declaration it cannot read a
  contract from, or a name declared twice (`Invalid`, naming both sources).
- `Io` for a root that is not a directory, and for a failed publication.
- `Serde` when JSON rendering fails; an unsupported extension or format name is `Invalid`.

Run `cargo test --locked -p xrf-export` from the repository root. See the [parser](src/extern_parser.rs),
[manifest](src/extern_manifest.rs) and [rendering](src/render.rs).
