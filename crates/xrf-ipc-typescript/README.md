# xrf-ipc-typescript

Generate the TypeScript mirror of a Tauri command surface and the Rust types it carries.

## Generate a mirror

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

An [`IpcCommandSurface`] is one plugin: its `tauri_specta::Builder`, and the raw commands that builder cannot hold.
The order the steps run in belongs to the generator and is not a caller's to reproduce — commands are exported first
because that is what collects the types they reference, the type modules are written next because only then is it known
which module declares what, and each command module is finalized last because the type modules say which imports
replace the copies Specta inlined.

## What it writes

Output splits into `types/` and `commands/`. A type is written once, into the module of the crate that declares it,
read off `module_path!()` rather than from any hand-written list; `TypeOwnership` resolves that mapping, renders the
imports tying the modules together, and rejects an import cycle. Which commands exist, and which application they
belong to, is never known here — the caller supplies the Specta builders and the output directory.

`finalize_command_module` rewrites Specta's `invoke` import to the counted one and asserts the rewrite happened, so
IPC counting cannot silently stop. `export_raw_commands` writes wrappers for commands that return bytes and bypass
Specta.

## Enumerations

Every exported Rust enum gains a TypeScript enum, so a frontend comparison names a member the compiler resolves
instead of quoting a spelling nothing checks. `EnumerationSubject` decides which of two shapes a type is:

- A closed set of identities gets an enum **replacing** the union, with the derived `${E..}` union beside it. Only
  these may stand in for the type itself, which is what a command _parameter_ is rewritten to. A return keeps the
  union: what arrives is a raw string off the IPC channel, and naming it as a nominal enum would assert about it
  rather than check it.
- An internally tagged union keeps its object shapes and gains an enum of its **discriminants**, declared before it.
  That enum stands in for the tag and never for the value.

Tagging is detected from the mapped shape — an internally tagged variant renders as named fields whose tag field is a
one-member literal carrying that variant's own spelling — because `specta-serde` keeps `EnumRepr` private. Externally
tagged, adjacently tagged and untagged unions fall out on their own, having no discriminant to name.

A member name comes from the Rust identifier rather than the wire spelling, which carries no word boundaries:
upper-casing `AiCrow`'s spelling gives `AICROW` where the identifier gives `AI_CROW`. Two variants folding onto one
member name fail loudly, naming the Rust identifiers, because renaming one of those is the only fix.

A consumer should know one caveat: an enum member `case` narrows a union in TypeScript but does not prove it
exhausted, so a `switch` written with members needs an `assertExhaustive`-style default or a variant added in Rust
compiles silently.

## Verifying committed mirrors

`read_surface` reads every exported declaration of a generated tree into a canonical form, and `compare_surfaces`
reports the drift between a committed tree and a fresh generation. `SurfaceDrift::is_breaking` is true when a
declaration was removed or changed shape; an addition alone is reported as context. That is what lets a consumer keep
its mirrors in version control and fail a test when the Rust sources move past them.

Run `cargo test --locked -p xrf-ipc-typescript`.

## Layout

One type per file, named for it: [`IpcBindingsGenerator`](src/ipc_bindings_generator.rs) owns the run,
[`TypeModuleWriter`](src/type_module_writer.rs) and [`CommandModuleWriter`](src/command_module_writer.rs) write the two
directories, [`TypeOwnership`](src/type_ownership.rs) answers which module declares what, and
[`Surface`](src/bindings_surface.rs) reads a committed tree back.

[`typescript/`](src/typescript/) is vocabulary rather than a layer: `source` reads generated text, `syntax` emits it,
`normalization` canonicalizes a parsed declaration, and `format` is the Specta shape the frontend expects. Every module
in it is a leaf, depended on from above and depending on nothing else here, which is why it is free functions rather
than types. `generated_output` is the one other such module, writing a file the frontend toolchain will reformat.

The rest is one layer, not two: the writers of `types/` and `commands/` share ownership, enumerations, output and the
exporter, so there is no `types/` and `commands/` split to make.
