# xrf-typescript

Parses TypeScript with SWC and resolves the contracts of exported symbols, for the tools that read the engine's
TypeScript declarations: [xrf-export](../xrf-export/README.md) extracts XRF externs with it, and
[xrf-ipc-typescript](../xrf-ipc-typescript/README.md) reads generated bindings back to compare them. It reads syntax
only; it is not a type checker.

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

`parse_typescript_file` reads the path from disk and returns a `TypeScriptSource`. Keep its source map with the AST:
spans resolve to file positions only through it. SWC holds comments apart from the program tree, so they come back
separately. The crate re-exports `swc_common` and `swc_ecma_ast` so a consumer inspects the tree with the same versions.

| Value                          | Contents                             |
| ------------------------------ | ------------------------------------ |
| `TypeScriptSource::program`    | Parsed module or script              |
| `TypeScriptSource::comments`   | Comments indexed by source positions |
| `TypeScriptSource::source_map` | File and span information            |

The parser uses SWC's TypeScript syntax with its default options, so JSX (`.tsx`) is not accepted. A file that cannot be
read is `XrfError::Io`. Any syntax diagnostic is `XrfError::Parsing`, listing each as `path:line:column: message`; that
includes diagnostics the parser recovered from, so a program accompanied by syntax errors is never returned as clean.

The `ast` module holds the helpers the consumers share: `expression_callee_name` and `expression_string_argument` read a
call, and `ts_type_to_string` renders a type node as stable TypeScript-like text (`Array<T>`, unions parenthesized).
Unsupported nodes render as `unsupported` with a logged warning; `canonical_ts_type_to_string` falls back to the source
snippet, whitespace collapsed, for those and for type literals and import types.

## Resolve symbols

`TypeScriptSymbolResolver::discover(root)` reads the nearest `tsconfig.json` in `root` or its ancestors, as JSONC, and
takes only `compilerOptions.baseUrl` and `compilerOptions.paths` from it; `extends` is not followed. With no
configuration it resolves relative imports only. An unreadable configuration is `XrfError::Io`, a malformed one
`XrfError::Invalid`.

`resolve_symbol(source_file, source_map, program, name)` answers a `TypeScriptSymbol` for a name used in a parsed file:

1. A declaration in the file itself, exported or not.
2. Otherwise a named import of it; a specifier marked `type` is skipped. The module specifier resolves relative to the
   importing file when it starts with `.`, or through the most specific matching `paths` alias (longest fixed prefix,
   then suffix), trying the alias's targets in configured order, relative to `baseUrl` or else the configuration's
   directory. Each candidate is tried as written, with `.ts`, then as `<dir>/index.ts`. A bare package name no alias
   matches does not resolve.
3. In the imported module, an exported declaration, then named and wildcard re-exports, followed recursively. A
   re-export cycle ends unresolved.

| Symbol     | Declared as                                                                            |
| ---------- | -------------------------------------------------------------------------------------- |
| `Callable` | A function declaration, or a variable initialized with an arrow or function expression |
| `Value`    | A variable with a type annotation, rendered canonically                                |
| `Object`   | An unannotated object literal, with its identifier- or string-keyed properties         |

`as const`, `satisfies` and parentheses around an initializer are looked through. An object property's type is inferred
from functions, nested objects, `as T`, and boolean, number or string literals; anything else is `unknown`.

It answers `Ok(None)` when no supported declaration resolves, which differs from an error reading or parsing a module
it reached. `resolve_member_type` resolves an object the same way and answers the type of one named property.

## Canonical rendering

The optional `codegen` feature enables the `renderer` module. It is off by default so parsing-only consumers (the CLI,
through `xrf-export`) do not compile the emitter.

```toml
[dependencies]
xrf-typescript = { workspace = true, features = ["codegen"] }
```

With that feature, `render_module_item` renders one SWC module item through SWC's minifying emitter: one canonical
spelling, without the original comments or layout. Use it to compare declarations independently of their formatting,
not to preserve a user's source text for an edit.

## Checks

Run `cargo test --locked -p xrf-typescript`, adding `--features codegen` to test the renderer. See the
[parser](src/parser.rs), [AST helpers](src/ast.rs), [symbol resolver](src/symbol_resolver), and
[renderer](src/renderer.rs).
