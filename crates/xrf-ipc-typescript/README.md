# xrf-ipc-typescript

Generate the TypeScript mirror of a Tauri command surface and the Rust types it carries.

## Generate a mirror

```rust,no_run
use std::path::Path;
use std::sync::{Arc, Mutex};

use specta::Types;
use xrf_ipc_typescript::{COMMANDS_DIRECTORY, TYPES_DIRECTORY, command_exporter, export_type_modules, reset_directory};

fn export(output: &Path, builder: &tauri_specta::Builder<tauri::Wry>) {
  let collected: Arc<Mutex<Types>> = Arc::new(Mutex::new(Types::default()));

  reset_directory(&output.join(TYPES_DIRECTORY));
  reset_directory(&output.join(COMMANDS_DIRECTORY));

  builder
    .export(
      command_exporter(Arc::clone(&collected)),
      output.join(COMMANDS_DIRECTORY).join("archives.ts"),
    )
    .expect("commands are exported");

  let collected: Types = Arc::try_unwrap(collected).expect("nothing still holds the types").into_inner().expect("the lock is intact");

  export_type_modules(&output.join(TYPES_DIRECTORY), &collected);
}
```

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

See [enumerations](src/enumerations/), [type ownership](src/ownership.rs), and [surface drift](src/surface.rs).
