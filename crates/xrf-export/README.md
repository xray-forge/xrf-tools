# xrf-export

Extracts XRF TypeScript `extern(...)` declarations into a manifest and renders JSON, XML, or HTML documentation.

## Extract and render declarations

```rust,no_run
use std::path::Path;
use xrf_export::{ExternFormat, ExternManifestParser, render_extern_manifest};

fn main() -> xrf_error::XrfResult {
  let parsed = ExternManifestParser::new().parse_directory(Path::new("src/declarations"))?;
  let json = render_extern_manifest(&parsed.manifest, ExternFormat::Json, None)?;

  println!("{} exports", parsed.manifest.exports.len());
  print!("{json}");

  Ok(())
}
```

The input directory must contain XRF extern declarations. Parsing reads TypeScript source and its contracts; it does
not execute declarations or require the game's Lua runtime.

## Manifest and project views

Call `ExternManifestParser::new().parse_directory` with the declarations root. The returned `ParsedExternManifest`
contains the canonical manifest and parsed declaration details. Source paths are relative to that root.

`ExternManifest` maps exported names to `ExternExport::Callable` or `ExternExport::Value`. A callable carries parameter
and return contracts; a value carries its asserted type. Both retain the source and optional documentation.

Use the canonical manifest for generated artifacts and `ExportsProjectParser` when a browser also needs source and
contract descriptors. Keep relative source paths rooted at the same declarations directory when linking diagnostics.

## Output formats

Pass the manifest to `render_extern_manifest` with an `ExternFormat`. An explicit line-ending choice overrides the
format's default.

| Format | Typical use                 | Default endings |
| ------ | --------------------------- | --------------- |
| `Json` | Machine-readable manifest   | CRLF            |
| `Xml`  | Structured export document  | LF              |
| `Html` | Readable contract reference | LF              |

Rendering returns a string with a final newline; it does not write a file. `write_extern_manifest` publishes a rendered
manifest through staged file output, creating parent directories. `ExternFormat::from_extension` accepts `.json`,
`.xml`, `.html`, and `.htm`; unsupported extensions return an error.

## Source selection and limitations

The parser selects eligible `.ts` files containing extern calls, excluding test files and generated/dependency
directories. It is an extractor for XRF's declaration contract, not a general TypeScript documentation generator.
Duplicate extern names are rejected.

Declarations need to fit the supported extern contract. A declaration's documentation does not replace its type
information, and successful extraction does not prove that the exported implementation is callable in the game.

## Errors and checks

Malformed selected declarations and duplicate names fail parsing or validation. An invalid root, failed read, or
failed publication can return an I/O error. Rendering JSON can return a serialization error. Preserve these causes
when reporting why an artifact could not be generated.

Run `cargo test --locked -p xrf-export` from the repository root. TypeScript parsing and symbol resolution are provided
by [xrf-typescript](../xrf-typescript/README.md).

See the [parser](src/extern_parser.rs), [manifest](src/extern_manifest.rs), and [rendering API](src/render.rs).
