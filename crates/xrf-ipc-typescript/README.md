# xrf-ipc-typescript

Generates the frontend's TypeScript mirror of a Tauri command surface and the Rust types it carries, and checks a
committed mirror against the Rust sources.

The crate knows no particular application. The caller supplies:

- each plugin's Specta builder;
- the output directory;
- which crates share a types module (`TypeModules`);
- how generated modules import one another and the runtime beside them (`MirrorImports`).

The desktop app drives it from `bin/xrf-app/app/src/ipc/bindings/generator.rs` and writes `packages/xrf-ipc/src/`.
`cargo make generate-typescript` regenerates that tree and formats it.

## Generate and verify

```rust,no_run
use std::path::Path;

use xrf_error::XrfResult;
use xrf_ipc_typescript::{IpcBindingsGenerator, IpcCommandSurface, MirrorImports, SurfaceDrift, TypeModules};

const IMPORTS: MirrorImports = MirrorImports::new("#");

fn generate<R: tauri::Runtime>(output: &Path, surfaces: &[IpcCommandSurface<R>]) -> XrfResult {
  IpcBindingsGenerator::generate(output, surfaces, &TypeModules::default(), &IMPORTS)
}

fn verify<R: tauri::Runtime>(committed: &Path, scratch: &Path, surfaces: &[IpcCommandSurface<R>]) -> XrfResult {
  let drift: SurfaceDrift =
    IpcBindingsGenerator::verify(committed, scratch, surfaces, &TypeModules::default(), &IMPORTS)?;

  assert!(!drift.is_breaking(), "{}", drift.describe());

  Ok(())
}
```

The app runs generation as an ignored `#[test]`, because the test harness is what links the Specta builders in.

### Inputs

An `IpcCommandSurface` is one plugin:

- its name, which is the wire namespace and the command module's file stem;
- its `tauri_specta::Builder`;
- the raw commands and bulk routes the builder cannot hold, each declared by hand as a name and
  `(argument, TypeScript type)` pairs.

`MirrorImports::new(root)` names the output directory as its modules import it: `@/core/ipc` under an application
alias, or `#` inside a package that maps `"#/*"` to its own sources. The desktop app's mirror is the `@xrf/ipc` package,
generated with the root `#`.

The runtime beside the mirror must provide:

| Module           | Provides                                                         |
| ---------------- | ---------------------------------------------------------------- |
| `invoke`         | The counting `invoke`                                            |
| `raw`            | `invokeRaw`                                                      |
| `bulk/bulk-call` | `IBulkCall` (a file, because `#/*` resolves no directory barrel) |
| `signature`      | `IIpcCommandSignature`, `IIpcBulkRouteSignature`                 |

### What `generate` does

It deletes and rewrites `types/` and `commands/` under `output`, in a fixed order:

1. Export the commands, collecting the types they reference and rendering their wrappers.
2. Write the type modules, which settles which module declares what.
3. Write each command module, importing from the type modules.
4. Write the merged signatures, which import every command module.

It fails when the output cannot be written or an invariant breaks:

- a type owned by two modules, or type modules importing each other;
- a referenced type declared outside the workspace;
- two surfaces or declarations with one name;
- a builder configured differently from what its wrappers are rendered for.

Whatever it wrote before failing stays for the next run to replace.

## What it writes

| Path                        | Contents                                                                          |
| --------------------------- | --------------------------------------------------------------------------------- |
| `types/xrf-<crate>.ts`      | Every exported type and constant one crate or family declares                     |
| `commands/<plugin>.ts`      | `E<Plugin>Command`, the typed commands as `<plugin>Commands`, their signatures    |
| `commands/<plugin>-raw.ts`  | The same for raw commands, only for a plugin declaring any                        |
| `commands/<plugin>-bulk.ts` | `E<Plugin>BulkRoute`, `<plugin>BulkRoutes` and their signatures, where it has any |
| `commands/signatures.ts`    | `IIpcCommandSignatures` and `IIpcBulkRouteSignatures`, every plugin's merged      |

### Type modules

- A type is written once, into the module of the crate that declares it, taken from the first segment of its
  `module_path!()`: `xrf_renderer` becomes `xrf-renderer.ts`. There is no hand-written list.
- A family of crates can share one module: `TypeModules::with_family("xrf_app", "xrf-app")` puts `xrf_app` and every
  `xrf_app_<part>` into `xrf-app.ts`.
- Only `xrf_*` crates get modules. A command referencing a type declared elsewhere fails generation.
- Modules import one another as `<root>/types/<module>`.

### Command modules

`CommandWrappers` renders each wrapper from the builder's function data through Specta's exporter, never from text
Specta wrote:

```ts
(arguments) => __TAURI_INVOKE<Return>(member, { arguments });
```

- This matches Tauri Specta for a builder that throws on error, keeps no serde phases, casts big integers to numbers,
  and declares no events, semantic types or constants. The generator asserts every builder is configured that way.
- The object is `<plugin>Commands`; its `invoke` is the counting wrapper from `<root>/invoke`. `Channel` is imported
  from Tauri where a wrapper carries one.
- Raw command wrappers call `invokeRaw` from `<root>/raw` and return `Promise<ArrayBuffer>`; a raw command is a Tauri
  command answering `tauri::ipc::Response`.
- Bulk route wrappers are served outside IPC, so they return the call as data: an `IBulkCall` with its route and
  arguments.

A Specta upgrade therefore shows up as a compile error on its datatype API, not as output the generator silently
misreads.

### Command enums and signatures

Each module names what it wraps in a string enum, so code names a command by a checked member instead of its wire name:

- `EJobsCommand.ATTACH = "plugin:jobs|attach"`;
- `EArchivesBulkRoute.READ_ENTRY = "archives/read_entry"`;
- members are named after the Rust command: `open_level` is `OPEN_LEVEL`.

`I<Plugin>CommandSignatures` maps each member to
`IIpcCommandSignature<typeof <plugin>Commands.<wrapper>, [<argument names>]>`. The signature is read off the wrapper in
TypeScript, so it carries the wrapper's own types, retyped enum parameters included. Argument names come from Specta's
function data, camel-cased as Tauri Specta writes them.

`signatures.ts` merges every plugin's command and raw signatures into `IIpcCommandSignatures`, and every bulk route's
into `IIpcBulkRouteSignatures`; a typed mock keys by member.

### Constants

A constant marked `#[xrf_ipc::constant]` (see [xrf-ipc](../xrf-ipc/README.md)) lands in its crate's module by the same
rule. The generator reads every `xrf_ipc::IpcConstant` linked into the binary.

- Constants are written after the imports and before the types, sorted by name, each under its doc comment as JSDoc.
- A crate with constants and no types still gets a module.
- Each constant's type joins the collected types, so a type only a constant carries is declared in its crate's module.

The TypeScript type follows the serialized value:

| Value           | Written as                                                                                                     |
| --------------- | -------------------------------------------------------------------------------------------------------------- |
| Number          | `export const NAME: number = 4;` (an `f32` in its shortest form)                                               |
| String          | Its literal: `export const ARCHIVES_JOB_PACK = "archives.pack" as const;`                                      |
| Boolean         | `boolean`                                                                                                      |
| Object or array | Its Specta type, as a literal with bare keys: `export const ARCHIVE_READ_POLICY: ArchiveReadPolicy = { ... };` |

String constants keep their literal type, so a kind of value is the union of its constants.

Generation fails, naming the constant, on:

- a value that does not serialize, or a bare `null`;
- an integer beyond ±(2^53−1) at any depth;
- a type Specta cannot name;
- two constants with one name in a module, or a constant named like a type in its module.

## Enumerations

Every exported Rust enum also gets a TypeScript enum, `E<Type>`, so frontend comparisons use checked members instead of
quoted spellings. There are two shapes:

- **Variants without data** get an enum that replaces the union, plus a union derived from it. A command parameter is
  rewritten to the enum. A return keeps the union: what arrives is a raw string off IPC, and typing it as the enum
  would assert rather than check.
- **Internally tagged unions** keep their object shapes and gain an enum of their tags, declared before them. The enum
  types the tag, never the value.

For `enum Shading { Flat, PerPixel }` serialized in camelCase:

```ts
export enum EShading {
  FLAT = "flat",
  PER_PIXEL = "perPixel",
}

/** Every `EShading` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type Shading = `${EShading}`;
```

- Tagging is detected from the shape: an internally tagged variant renders as named fields with a one-member literal
  tag. (`specta-serde` keeps `EnumRepr` private.) Externally tagged, adjacently tagged and untagged unions have no tag
  to name and get nothing.
- Member names come from the Rust identifier, not the wire spelling: `AiCrow` gives `AI_CROW`, not `AICROW`. Two
  variants that fold to one name fail generation, naming both; rename one.
- A variant's doc comment carries over to its member.

Caveat for consumers: a `switch` over enum members narrows the union but does not prove it exhaustive. Add an
`assertExhaustive`-style default, or a variant added in Rust compiles silently.

## Verify a committed mirror

`IpcBindingsGenerator::verify` generates into a scratch tree, parses every exported declaration of both trees into a
canonical form (so the frontend formatter's layout does not matter), and compares them.

- `SurfaceDrift::is_breaking` is true when a declaration was removed or changed shape. Additions alone are listed in
  `describe` as context.
- `verify` fails on anything `generate` fails on, and when either tree does not parse as TypeScript.

This lets a consumer keep its mirror in version control and fail a test when the Rust sources move past it.

## Layout

One type per file, named for it. [`IpcBindingsGenerator`](src/ipc_bindings_generator.rs) runs the steps above; each
lives in its own directory:

| Directory                            | Holds                                                                                                                                                                                                              |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`input/`](src/input/)               | What the caller passes: `IpcCommandSurface`, `TypeModules`, `MirrorImports`                                                                                                                                        |
| [`collected/`](src/collected/)       | What the run gathers first: `CommandTypescript` records each plugin's `PluginCommands` (its `SpectaCommand`s and their `CommandWrappers`) into `CollectedCommands`; `CollectedConstants` adds the marked constants |
| [`write/`](src/write/)               | `TypeModuleWriter` and `CommandModuleWriter` writing through `GeneratedOutput`; `TypeOwnership`, `PluginSymbols`, `PluginModules`                                                                                  |
| [`verify/`](src/verify/)             | `Surface` reads a generated tree back; `SurfaceDrift` compares two                                                                                                                                                 |
| [`enumerations/`](src/enumerations/) | The `E*` enums: `Enumeration` and `EnumerationSubject`                                                                                                                                                             |
| [`typescript/`](src/typescript/)     | Leaf helpers: `source` reads generated text, `syntax` emits it, `normalization` canonicalizes, `format` is the expected Specta shape                                                                               |

## Checks

Run `cargo test --locked -p xrf-ipc-typescript` from the repository root.
