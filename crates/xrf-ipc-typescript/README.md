# xrf-ipc-typescript

Generates the TypeScript mirror of a Tauri command surface and the Rust types it carries, and checks a committed mirror
against the Rust sources. The crate knows nothing of any one application: the caller supplies each plugin's Specta
builder and the output directory. The desktop app drives it from `bin/xrf-app/src/ipc/bindings/generator.rs`, writing
`bin/xrf-ui/src/core/ipc/`; `cargo make generate-typescript` regenerates and then formats that tree.

## Generate and verify

```rust,no_run
use std::path::Path;

use xrf_ipc_typescript::{IpcBindingsGenerator, IpcCommandSurface, SurfaceDrift};

fn generate<R: tauri::Runtime>(output: &Path, surfaces: &[IpcCommandSurface<R>]) {
  IpcBindingsGenerator::generate(output, surfaces);
}

fn verify<R: tauri::Runtime>(committed: &Path, scratch: &Path, surfaces: &[IpcCommandSurface<R>]) {
  let drift: SurfaceDrift = IpcBindingsGenerator::verify(committed, scratch, surfaces);

  assert!(!drift.is_breaking(), "{}", drift.describe());
}
```

An `IpcCommandSurface` is one plugin: its name (the wire namespace and the command module's file stem), its
`tauri_specta::Builder`, and the raw commands and bulk routes that builder cannot hold, each declared by hand as a name
and `(argument, TypeScript type)` pairs.

`generate` deletes and rewrites `types/` and `commands/` under `output`. It panics rather than returning an error: when
the output cannot be written, or when an invariant the mirror rests on breaks (a type owned by two modules, type
modules importing each other, a referenced type declared outside the workspace, an `invoke` that bypasses the counted
wrapper). The app runs it as an ignored `#[test]`, because the test harness is what links the Specta builders in.

The order of its steps belongs to the generator, not the caller: commands are exported first, because that collects the
types they reference; the type modules are written next, because only then is it known which module declares what; and
each command module is finalized last, because the type modules say which imports replace the copies Specta inlined.

## What it writes

| Path                        | Contents                                                        |
| --------------------------- | --------------------------------------------------------------- |
| `types/xrf-<crate>.ts`      | Every exported type and constant one workspace crate declares   |
| `commands/<plugin>.ts`      | The plugin's typed commands as `export const <plugin>Commands`  |
| `commands/<plugin>-raw.ts`  | `<plugin>RawCommands`, only for a plugin declaring raw commands |
| `commands/<plugin>-bulk.ts` | `<plugin>BulkRoutes`, only for a plugin declaring bulk routes   |

A type is written once, into the module of the crate that declares it, read off the first segment of its
`module_path!()` (`xrf_renderer` becomes `xrf-renderer.ts`) rather than from any hand-written list. Only `xrf_*` crates
have modules; a command referencing a type declared anywhere else fails generation. Modules import one another as
`@/core/ipc/types/<module>`, the frontend's path alias.

A command module keeps Specta's command object, renamed from `commands` to `<plugin>Commands` in camelCase, with the
inlined type copies cut and replaced by imports. Its `invoke` import is redirected from `@tauri-apps/api/core` to the
counting wrapper `@/core/ipc/invoke` (`Channel` still comes from Tauri), and generation fails if Specta's import
spelling changes so that the redirect no longer applies. Raw command wrappers call `invokeRaw` from `@/core/ipc/raw`
and return `Promise<ArrayBuffer>`, a raw command being a Tauri command answering `tauri::ipc::Response`. Bulk route
wrappers are served outside IPC, so they answer the call as data, an `IBulkCall` holding its route and arguments.

A constant marked `#[xrf_ipc::constant]` where it is declared (see [xrf-ipc](../xrf-ipc/README.md)) lands in its
crate's module by the same rule. The generator reads every `xrf_ipc::IpcConstant` linked into the binary and writes a
module's constants after its imports and before its types, sorted by name, each under its doc comment as JSDoc:
`export const NAME: number = 4;`. A crate with constants and no types still gets a module. The TypeScript type follows
the serialized value: `number` (an `f32` in its own shortest form), `string` or `boolean`. Generation fails, naming the
constant, on a value that does not serialize or is anything else, an integer past ±(2^53−1), two constants of one name
in a module, or a constant named as a type of its module.

## Enumerations

Every exported Rust enum gains a TypeScript enum named `E<Type>`, so a frontend comparison names a member the compiler
resolves instead of quoting a spelling nothing checks. One of two shapes applies:

- A closed set of identities (variants carrying no data) gets an enum **replacing** the union, with the union derived
  from it beside it. Only these may stand in for the type itself, which is what a command _parameter_ is rewritten to.
  A return keeps the union: what arrives is a raw string off the IPC channel, and naming it as a nominal enum would
  assert about it rather than check it.
- An internally tagged union keeps its object shapes and gains an enum of its **discriminants**, declared before it.
  That enum stands in for the tag and never for the value.

For `enum Shading { Flat, PerPixel }` serialized in camelCase, the generator writes:

```ts
export enum EShading {
  FLAT = "flat",
  PER_PIXEL = "perPixel",
}

/** Every `EShading` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type Shading = `${EShading}`;
```

Tagging is detected from the mapped shape (an internally tagged variant renders as named fields whose tag field is a
one-member literal carrying that variant's own spelling), because `specta-serde` keeps `EnumRepr` private. Externally
tagged, adjacently tagged and untagged unions have no discriminant to name and gain nothing.

A member name comes from the Rust identifier rather than the wire spelling, which carries no word boundaries:
upper-casing `AiCrow`'s spelling gives `AICROW` where the identifier gives `AI_CROW`. Two variants folding onto one
member name fail generation, naming the Rust identifiers, because renaming one of those is the only fix. A variant's
doc comment travels onto its member.

A consumer should know one caveat: an enum member `case` narrows a union in TypeScript but does not prove it exhausted,
so a `switch` written with members needs an `assertExhaustive`-style default, or a variant added in Rust compiles
silently.

## Verifying committed mirrors

`IpcBindingsGenerator::verify` generates into a scratch tree, reads every exported declaration of both trees into a
canonical form (parsed and re-emitted, so the frontend formatter's layout does not count), and compares them.
`SurfaceDrift::is_breaking` is true when a declaration was removed or changed shape; an addition alone is listed in
`describe` as context. That lets a consumer keep its mirror in version control and fail a test when the Rust sources
move past it. `verify` panics on anything `generate` does, and when either tree does not parse as TypeScript.

Run `cargo test --locked -p xrf-ipc-typescript`.

## Layout

One type per file, named for it: [`IpcBindingsGenerator`](src/ipc_bindings_generator.rs) owns the run,
[`TypeModuleWriter`](src/type_module_writer.rs) and [`CommandModuleWriter`](src/command_module_writer.rs) write the two
directories, [`TypeOwnership`](src/type_ownership.rs) answers which module declares what, and
[`Surface`](src/bindings_surface.rs) reads a generated tree back.

[`typescript/`](src/typescript/) is vocabulary rather than a layer: `source` reads generated text, `syntax` emits it,
`normalization` canonicalizes a parsed declaration, and `format` is the Specta shape the frontend expects. Every module
in it is a leaf, depended on from above and depending on nothing else here, which is why it is free functions rather
than types. `generated_output` is the one other such module, writing a file the frontend toolchain will reformat.

The rest is one layer, not two: the writers of `types/` and `commands/` share ownership, enumerations, output and the
exporter, so there is no `types/` and `commands/` split to make.
