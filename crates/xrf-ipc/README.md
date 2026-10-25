# xrf-ipc

Marks a constant for the frontend's generated bindings where it is declared, as `#[derive(specta::Type)]` marks a
type, so no list of exported constants exists anywhere else. The marker records the constant in the binary;
[xrf-ipc-typescript](../xrf-ipc-typescript/README.md) reads every record and writes it into the TypeScript mirror.

## Export a constant

`#[xrf_ipc::constant]` on a module-level `const` item keeps the item as written and registers an `IpcConstant`: the
declaring module's path, the name, the `///` doc comment, a function serializing the value, and its type's Specta
definition. `IpcConstant::list()` iterates every constant registered in the binary, in no particular order.

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

Declared in `xrf-renderer`, that constant is generated into `core/ipc/types/xrf-renderer.ts` under its doc as
`export const RENDER_MAX_SHADOW_FILTER: number = 3;`. The attribute's own rules and refusals are in
[xrf-ipc-derive](../xrf-ipc-derive/README.md).

## Caller obligations

- Depend on `xrf-ipc` itself: the expansion names `::xrf_ipc`, and this crate re-exports `inventory` and `serde_json`
  so a marking crate needs nothing else. Mark constants only while the crate emits bindings, with `xrf-ipc` an
  optional dependency its `typescript-bindings` feature enables:

  ```rust,ignore
  #[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
  pub const RENDER_MAX_SHADOW_FILTER: u32 = 3;
  ```

- A constant reaches the bindings only from a binary that links its crate with that feature on, as the app's
  `typescript-bindings` feature does for every crate it mirrors.
- Declare it in an `xrf_*` crate. Its bindings module is named after the crate, and generation fails on a constant
  declared anywhere else.
- Its type must implement `serde::Serialize` and `specta::Type`, the second as the crate's `typescript-bindings`
  feature derives it. The value is what `serde_json` writes of it, an `f32` in its own shortest form (`to_value` goes
  through the text, since `serde_json::to_value` would widen `0.1_f32` to `0.10000000149011612`). The generator takes
  numbers, strings, booleans, objects and arrays, integers only up to ±(2^53−1), the most a JavaScript number holds
  exactly; a non-finite float serializes as `null`, and a bare `null` is refused. An object or an array is typed by its
  Specta definition, which also declares a named type no command carries:
  `xrf_archive::ARCHIVE_READ_POLICY` is the frontend's only source of `ArchiveReadPolicy`.

`to_value`, `data_type_of` and `parent_module_path` are public only because the expansion calls them. The type is read
off the value, as the serialization is, so the expansion never restates a type the item spelled.

## Checks

Run `cargo test --locked -p xrf-ipc`. Sources: [`IpcConstant`](src/ipc_constant.rs), [`to_value`](src/to_value.rs).
