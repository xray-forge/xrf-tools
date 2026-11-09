# xrf-ipc

Marks a constant for the frontend's generated bindings where it is declared, the way `#[derive(specta::Type)]` marks a
type. No separate list of exported constants exists.

The marker records the constant in the binary; [xrf-ipc-typescript](../xrf-ipc-typescript/README.md) reads every record
and writes it into the TypeScript mirror.

## Export a constant

`#[xrf_ipc::constant]` on a module-level `const` keeps the item as written and registers an `IpcConstant` holding:

- the declaring module's path and the name;
- the `///` doc comment;
- a function serializing the value;
- its type's Specta definition.

`IpcConstant::list()` iterates every registered constant in the binary, in no particular order.

```rust
/// Texels the sun's filter reaches each way at most.
#[xrf_ipc::constant]
pub const RENDER_MAX_SHADOW_FILTER: u32 = 3;

fn main() {
  let exported = xrf_ipc::IpcConstant::list()
    .find(|it| it.name == "RENDER_MAX_SHADOW_FILTER")
    .unwrap();

  assert_eq!(exported.doc, "Texels the sun's filter reaches each way at most.");
  assert_eq!((exported.value)().unwrap(), xrf_ipc::serde_json::json!(3));
}
```

Declared in `xrf-renderer`, that constant is generated into `core/ipc/types/xrf-renderer.ts`, under its doc, as
`export const RENDER_MAX_SHADOW_FILTER: number = 3;`. The attribute's own rules are in
[xrf-ipc-derive](../xrf-ipc-derive/README.md).

## Requirements

**Dependency.** Depend on `xrf-ipc` itself: the expansion names `::xrf_ipc`, and this crate re-exports `inventory` and
`serde_json`, so nothing else is needed. Mark constants only while the crate emits bindings, with `xrf-ipc` an optional
dependency enabled by its `typescript-bindings` feature:

```rust,ignore
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAX_SHADOW_FILTER: u32 = 3;
```

**Linking.** A constant reaches the bindings only from a binary that links its crate with that feature on, as the app's
`typescript-bindings` feature does for every crate it mirrors.

**Location.** Declare it in an `xrf_*` crate. Its bindings module is named after the crate, and generation fails for a
constant declared anywhere else.

**Type.** It must implement `serde::Serialize` and `specta::Type` (the crate's `typescript-bindings` feature derives the
latter).

- The value is what `serde_json` writes; an `f32` is written in its own shortest form.
- Numbers, strings, booleans, objects and arrays are accepted. Integers must fit in ±(2^53−1), the most a JavaScript
  number holds exactly.
- A non-finite float serializes as `null`; a bare `null` is refused.
- An object or array is typed by its Specta definition. That can also declare a type no command carries:
  `xrf_archive::ARCHIVE_READ_POLICY` is the frontend's only source of `ArchiveReadPolicy`.

## Hidden helpers

`IpcConstant::serialize_value`, `define_type` and `enclosing_module` are public but hidden from the docs, because only
the expansion calls them.

- `serialize_value` goes through text, since `serde_json::to_value` would widen `0.1_f32` to `0.10000000149011612`.
- The type is read off the value, so the expansion never restates the type the item spelled.

## Checks

Run `cargo test --locked -p xrf-ipc`. Source: [`IpcConstant`](src/ipc_constant.rs).
