# xrf-ipc-typescript

Generates the TypeScript mirror of a Tauri command surface and the Rust types it carries, and checks a committed mirror
against the Rust sources. The crate knows nothing of any one application: the caller supplies each plugin's Specta
builder, the output directory, which crates share a types module, and how the generated modules address one another and
the runtime beside them (`MirrorImports`). The desktop app drives it from `bin/xrf-app/app/src/ipc/bindings/generator.rs`,
writing `packages/xrf-ipc/src/`; `cargo make generate-typescript` regenerates and then formats that tree.

## Generate and verify

```rust,no_run
use std::path::Path;

use xrf_ipc_typescript::{IpcBindingsGenerator, IpcCommandSurface, MirrorImports, SurfaceDrift, TypeModules};

const IMPORTS: MirrorImports = MirrorImports::new("#");

fn generate<R: tauri::Runtime>(output: &Path, surfaces: &[IpcCommandSurface<R>]) {
  IpcBindingsGenerator::generate(output, surfaces, &TypeModules::default(), &IMPORTS);
}

fn verify<R: tauri::Runtime>(committed: &Path, scratch: &Path, surfaces: &[IpcCommandSurface<R>]) {
  let drift: SurfaceDrift = IpcBindingsGenerator::verify(committed, scratch, surfaces, &TypeModules::default(), &IMPORTS);

  assert!(!drift.is_breaking(), "{}", drift.describe());
}
```

`MirrorImports::new(root)` names the output directory as its modules import it: `@/core/ipc` under an application's
alias, `#` inside a package mapping `"#/*"` to its own sources. The runtime beside the mirror provides `invoke` (the
counted `invoke`), `raw` (`invokeRaw`), `bulk/bulk-call` (`IBulkCall`; a file, since a package's `#/*` resolves no
directory barrel) and `signature` (`IIpcCommandSignature`, `IIpcBulkRouteSignature`). The desktop app's mirror lives in
the `@xrf/ipc` package (`packages/xrf-ipc/src/`), generated with the root `#`.

An `IpcCommandSurface` is one plugin: its name (the wire namespace and the command module's file stem), its
`tauri_specta::Builder`, and the raw commands and bulk routes that builder cannot hold, each declared by hand as a name
and `(argument, TypeScript type)` pairs.

`generate` deletes and rewrites `types/` and `commands/` under `output`. It panics rather than returning an error: when
the output cannot be written, or when an invariant the mirror rests on breaks (a type owned by two modules, type
modules importing each other, a referenced type declared outside the workspace, an `invoke` that bypasses the counted
wrapper). The app runs it as an ignored `#[test]`, because the test harness is what links the Specta builders in.

The order of its steps belongs to the generator, not the caller: commands are exported first, because that collects the
types they reference and the commands themselves; the type modules are written next, because only then is it known which
module declares what; each command module is finalized after, because the type modules say which imports replace the
copies Specta inlined; and the merged signatures last, because they import every command module.

## What it writes

| Path                        | Contents                                                                         |
| --------------------------- | -------------------------------------------------------------------------------- |
| `types/xrf-<crate>.ts`      | Every exported type and constant one crate or family declares                    |
| `commands/<plugin>.ts`      | `E<Plugin>Command`, the typed commands as `<plugin>Commands`, their signatures   |
| `commands/<plugin>-raw.ts`  | The same for raw commands, only for a plugin declaring any                       |
| `commands/<plugin>-bulk.ts` | `E<Plugin>BulkRoute`, `<plugin>BulkRoutes` and their signatures, where it has any |
| `commands/signatures.ts`    | `IIpcCommandSignatures` and `IIpcBulkRouteSignatures`, every plugin's merged      |

A type is written once, into the module of the crate that declares it, read off the first segment of its
`module_path!()` (`xrf_renderer` becomes `xrf-renderer.ts`) rather than from any hand-written list. A caller may group a
family of crates into one module (`TypeModules::with_family("xrf_app", "xrf-app")` puts `xrf_app` and every
`xrf_app_<part>` into `xrf-app.ts`), so one surface split into crates stays one module to the frontend. Only `xrf_*`
crates have modules; a command referencing a type declared anywhere else fails generation. Modules import one another as
`<root>/types/<module>`, the root `MirrorImports` was given.

A command module keeps Specta's command object, renamed from `commands` to `<plugin>Commands` in camelCase, with the
inlined type copies cut and replaced by imports. Its `invoke` import is redirected from `@tauri-apps/api/core` to the
counting wrapper `<root>/invoke` (`Channel` still comes from Tauri), and generation fails if Specta's import spelling
changes so that the redirect no longer applies. Raw command wrappers call `invokeRaw` from `<root>/raw` and return
`Promise<ArrayBuffer>`, a raw command being a Tauri command answering `tauri::ipc::Response`. Bulk route wrappers are
served outside IPC, so they answer the call as data, an `IBulkCall` holding its route and arguments.

Each module names what it wraps in a string enum, so a caller or a test names a command by a member the compiler checks
instead of quoting its wire name: `EJobsCommand.ATTACH = "plugin:jobs|attach"`, `EArchivesBulkRoute.READ_ENTRY =
"archives/read_entry"`, members named after the Rust command (`open_level` is `OPEN_LEVEL`). The wrappers invoke the
members; generation fails when a wrapper Specta wrote does not invoke its wire name exactly once. Beside the wrappers,
`I<Plugin>CommandSignatures` keys each member to `IIpcCommandSignature<typeof <plugin>Commands.<wrapper>, [<argument
names>]>`: the record the command is invoked with and what it resolves to, read off the wrapper in TypeScript, so a
signature is the wrapper's own types, retyped enum parameters included. The argument names come from Specta's function
data, camel-cased as Tauri Specta writes them. `signatures.ts` merges every plugin's command and raw signatures into
`IIpcCommandSignatures` and every bulk route's into `IIpcBulkRouteSignatures`, which a typed mock keys by member.

A constant marked `#[xrf_ipc::constant]` where it is declared (see [xrf-ipc](../xrf-ipc/README.md)) lands in its
crate's module by the same rule. The generator reads every `xrf_ipc::IpcConstant` linked into the binary and writes a
module's constants after its imports and before its types, sorted by name, each under its doc comment as JSDoc:
`export const NAME: number = 4;`. A crate with constants and no types still gets a module. The TypeScript type follows
the serialized value: `number` (an `f32` in its own shortest form), the string's own literal (`export const
ARCHIVES_JOB_PACK = "archives.pack" as const;`, so a kind of value is the union of its constants) or `boolean`.
Generation fails, naming the
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
directories, [`TypeModules`](src/type_modules.rs) names a crate's module, [`MirrorImports`](src/mirror_imports.rs)
addresses it, [`TypeOwnership`](src/type_ownership.rs) answers which module declares what,
[`CollectedCommands`](src/collected_commands.rs) holds what the export collected, each command a
[`SpectaCommand`](src/specta_command.rs), [`PluginSymbols`](src/plugin_symbols.rs) spells a plugin's generated names,
and [`Surface`](src/bindings_surface.rs) reads a generated tree back.

[`typescript/`](src/typescript/) is vocabulary rather than a layer: `source` reads generated text, `syntax` emits it,
`normalization` canonicalizes a parsed declaration, and `format` is the Specta shape the frontend expects. Every module
in it is a leaf, depended on from above and depending on nothing else here, which is why it is free functions rather
than types. `generated_output` is the one other such module, writing a file the frontend toolchain will reformat.

The rest is one layer, not two: the writers of `types/` and `commands/` share ownership, enumerations, output and the
exporter, so there is no `types/` and `commands/` split to make.
