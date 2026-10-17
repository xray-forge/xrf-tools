# xrf-ipc

Marks a constant for the frontend's generated bindings where it is declared, as `#[derive(specta::Type)]` marks a
type: no list of exported constants exists anywhere else.

## Export a constant

`#[xrf_ipc::constant]` on a module-level `const` item registers it with its module path, name, doc comment and
serialized value. `xrf-ipc-typescript` writes every registered constant into its owning crate's types module, so the
constant below lands in `core/ipc/types/xrf-renderer.ts` as `export const RENDER_MAX_SHADOW_FILTER: number = 3;` under
its doc.

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

A crate marks its constants only while it emits bindings, with `xrf-ipc` an optional dependency its
`typescript-bindings` feature enables:

```rust,ignore
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAX_SHADOW_FILTER: u32 = 3;
```

The value is what `serde_json` makes of it; the generator takes numbers, strings and booleans. Registration goes
through `inventory`, re-exported here with `serde_json` so a marking crate depends on `xrf-ipc` alone; the attribute
comes from `xrf-ipc-derive`, and `to_value` and `parent_module_path` are what its expansion calls.

Run `cargo test --locked -p xrf-ipc`.
