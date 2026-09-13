# xrf-typescript

Parses TypeScript with SWC and provides AST helpers and symbol resolution for XRF declaration tooling.

## Parse a module

```rust,no_run
use std::path::Path;
use xrf_typescript::{parse_typescript_file, swc_ecma_ast::Program};

fn main() -> xrf_error::XrfResult {
  let source = parse_typescript_file(Path::new("src/example.ts"))?;

  match &source.program {
    Program::Module(module) => println!("{} module items", module.body.len()),
    Program::Script(script) => println!("{} statements", script.body.len()),
  }

  Ok(())
}
```

The path is read from disk. Keep the returned source map with the AST so diagnostics can resolve spans to file positions.
Comments are held separately from the program tree.

## Source and symbol models

Call `parse_typescript_file(&Path)` to obtain a `TypeScriptSource` with its program, comments, and source map. Use
`TypeScriptSymbolResolver` for project symbol lookup. SWC's common and AST types are re-exported for consumers that
need to inspect the tree.

| Value                          | Contents                                             |
| ------------------------------ | ---------------------------------------------------- |
| `TypeScriptSource::program`    | Parsed module or script                              |
| `TypeScriptSource::comments`   | Comments indexed by source positions                 |
| `TypeScriptSource::source_map` | File and span information                            |
| `TypeScriptSymbolResolver`     | Project-aware resolution of supported source symbols |

The symbol resolver discovers project configuration and follows supported module and declaration forms. It extracts
contracts for XRF tooling; it does not implement the TypeScript compiler's full type system.

`resolve_symbol` follows local declarations, named imports, configured aliases, and named or wildcard re-exports.
It returns `Ok(None)` when no supported declaration can be resolved; that differs from an error reading or parsing a
source. `resolve_member_type` asks for a named property's contract on a resolved object.

## Canonical rendering

The optional `codegen` feature enables the `renderer` module and its exports. It is off by default so parsing-only
consumers do not compile the emitter.

```toml
[dependencies]
xrf-typescript = { workspace = true, features = ["codegen"] }
```

With that feature, `render_module_item` renders one SWC module item using its source map. It emits a canonical spelling
without the original comments or layout. Use it when comparing declarations independently of their formatting, not
when preserving a user's source text for an edit.

## Errors and checks

File-read failures return `XrfError::Io`; syntax diagnostics return `XrfError::Parsing` with source positions.
Parsing is not TypeScript type checking.

Recovered parser diagnostics also fail the parse; an AST accompanied by syntax errors is not returned as a clean
result. Run `cargo test --locked -p xrf-typescript`, or add `--features codegen` to test the optional renderer.

See the [parser and tests](src/parser.rs), [symbol resolver](src/symbol_resolver), and
[renderer](src/renderer.rs). XRF extern extraction belongs to [xrf-export](../xrf-export/README.md).
