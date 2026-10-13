# xrf-ipc-derive

The `#[constant]` attribute `xrf-ipc` re-exports; depend on `xrf-ipc` and name it `xrf_ipc::constant`.

## `#[constant]`

On a `const` item, public or not, it keeps the item as written and registers an `xrf_ipc::IpcConstant` beside it:
`module_path!()`, the item's name, its `///` lines joined by `\n` and trimmed, and a function serializing its value.
It takes no arguments, and refuses anything but a `const` item, or a generic one, with the error on what it refuses.

```rust
/// Frames VBAO accumulates over at most.
#[xrf_ipc::constant]
const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: u32 = 32;

assert_eq!(RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION, 32);
```

```rust,compile_fail
#[xrf_ipc::constant]
static RENDER_MAX_SHADOW_FILTER: u32 = 3;
```

Run `cargo test --locked -p xrf-ipc-derive`; `tests/ui` holds the refused forms and their diagnostics.
