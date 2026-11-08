# xrf-typescript

Parses TypeScript with SWC and resolves the contracts of exported symbols. It reads syntax only; it is not a type
checker.

- [xrf-export](../xrf-export/README.md) extracts XRF externs with it.
- [xrf-ipc-typescript](../xrf-ipc-typescript/README.md) reads generated bindings back with it to compare them.

## Parse a module

```rust,no_run
use std::path::Path;
use xrf_typescript::TypeScriptSource;
use xrf_typescript::swc_ecma_ast::Program;

fn main() -> xrf_error::XrfResult {
  let source = TypeScriptSource::parse_file(Path::new("src/example.ts"))?;

  match &source.program {
    Program::Module(module) => println!("{} module items", module.body.len()),
    Program::Script(script) => println!("{} statements", script.body.len()),
  }

  Ok(())
}
```

`TypeScriptSource::parse_file` reads the path from disk.

- Keep the source map with the AST: spans resolve to file positions only through it.
- SWC keeps comments apart from the program tree, so they come back separately.
- `swc_common` and `swc_ecma_ast` are re-exported, so consumers inspect the tree with the same versions.

| Value                          | Contents                             |
| ------------------------------ | ------------------------------------ |
| `TypeScriptSource::program`    | Parsed module or script              |
| `TypeScriptSource::comments`   | Comments indexed by source positions |
| `TypeScriptSource::source_map` | File and span information            |

- The parser uses SWC's TypeScript syntax with default options, so JSX (`.tsx`) is not accepted.
- A file that cannot be read is `XrfError::Io`.
- Any syntax diagnostic, even one the parser recovered from, is `XrfError::Parsing`, listing each as
  `path:line:column: message`. A program with syntax errors is never returned as clean.

## Render types

`TypeScriptTypeRenderer` writes a type node as stable TypeScript-like text from its tree: `T[]` and `Array<T>` both as
`Array<T>`, unions and intersections parenthesized, string literals double-quoted.

| Function            | Answers                                                                                  |
| ------------------- | ---------------------------------------------------------------------------------------- |
| `render`            | The tree's text, each unsupported node as `unsupported`, with a logged warning           |
| `render_canonical`  | The tree's text, or the source text, whitespace collapsed, where any node is unsupported |
| `render_annotation` | `render_canonical` of an optional annotation, `unknown` where none is written            |

Type literals, mapped, conditional, indexed and template types, generic function types, and function-type parameters
without a name or annotation are unsupported. A source the map cannot give back renders as `unknown`.

## Resolve symbols

### Configuration

`TypeScriptSymbolResolver::discover(root)` reads the nearest `tsconfig.json` in `root` or its ancestors, as JSONC.

- Only `compilerOptions.baseUrl` and `compilerOptions.paths` are used; `extends` is not followed.
- With no configuration, only relative imports resolve.
- An unreadable configuration is `XrfError::Io`; a malformed one `XrfError::Invalid`.

### Lookup

`resolve_symbol(source_file, source_map, program, name)` returns a `TypeScriptSymbol` for a name used in a parsed file,
looking in this order:

1. **A declaration in the file itself**, exported or not.
2. **A named import of it.** A specifier marked `type` is skipped. The module specifier resolves:
   - relative to the importing file when it starts with `.`;
   - otherwise through the most specific matching `paths` alias (longest fixed prefix, then suffix), trying its targets
     in configured order, relative to `baseUrl` or else the configuration's directory.

   Each candidate is tried as written, then with `.ts` appended (`patrols.mock` is `patrols.mock.ts`), then as
   `<dir>/index.ts`. A bare package name no alias matches does not resolve.

3. **In the imported module:** an exported declaration, or a named export (`export { x as y }`) followed to the module
   it names or to the binding it exports. Only then wildcard re-exports, which an explicit export shadows. A module
   asked again for a name it is already resolving stops that path, which breaks re-export cycles.

| Symbol     | Declared as                                                                             |
| ---------- | --------------------------------------------------------------------------------------- |
| `Callable` | A function declaration, or a variable initialized with an arrow or function expression  |
| `Value`    | A variable with a type annotation, or initialized as `value as T`, rendered canonically |
| `Object`   | An unannotated object literal, with its identifier- or string-keyed properties          |

- `as const`, `satisfies` and parentheses around an initializer are looked through.
- An object property's type is inferred from functions, nested objects, `as T`, and boolean, number or string literals;
  anything else is `unknown`.
- `TypeScriptSymbol::type_name` writes any symbol as one type.
- `Ok(None)` means no supported declaration resolved, which differs from an error reading or parsing a module it
  reached.
- `resolve_member_type` resolves an object the same way and returns the type of one named property.

A signature's parameters come from `TypeScriptFunctionParameter::from_pattern`, which xrf-export also uses for inline
arrows. It accepts:

- an identifier;
- an array pattern, named as written up to its annotation;
- a rest identifier, typed by the array it collects;
- either of the first two with a default, which makes it optional.

Any other pattern is kept in place as `[parameter]: unknown`.

## Canonical rendering

The optional `codegen` feature enables `TypeScriptItemRenderer`. It is off by default so parsing-only consumers (the
CLI, through `xrf-export`) do not compile the emitter.

```toml
[dependencies]
xrf-typescript = { workspace = true, features = ["codegen"] }
```

With that feature, `TypeScriptItemRenderer::render` renders one SWC module item through SWC's minifying emitter: one
canonical spelling, without the original comments or layout. Use it to compare declarations independently of their
formatting, not to preserve a user's source text for an edit.

## Checks

Run `cargo test --locked -p xrf-typescript`, adding `--features codegen` to build the item renderer.

Sources: the
[source](src/typescript_source.rs), [type renderer](src/typescript_type_renderer.rs),
[symbol resolver](src/symbol_resolver), and [item renderer](src/typescript_item_renderer.rs).
