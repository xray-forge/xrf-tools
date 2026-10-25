# xrf-ipc-derive

The `#[constant]` attribute that [xrf-ipc](../xrf-ipc/README.md) re-exports. Depend on `xrf-ipc` and name it
`xrf_ipc::constant`; the expansion refers to `::xrf_ipc` and does not compile without it.

## `#[constant]`

On a module-level `const` item, public or private and of any name, it keeps the item as written and adds a hidden
module beside it, `__xrf_ipc_constant_<NAME>`, that submits an `xrf_ipc::IpcConstant` to `inventory`:

| Field         | Value                                                                                          |
| ------------- | ---------------------------------------------------------------------------------------------- |
| `module_path` | The module declaring the constant, not the hidden one (`IpcConstant::enclosing_module`)        |
| `name`        | The constant's identifier                                                                      |
| `doc`         | Its `///` lines, each without the one space after the slashes, joined by `\n`, then trimmed    |
| `value`       | A function serializing it through `IpcConstant::serialize_value`, so its type must `Serialize` |
| `data_type`   | Its type's definition through `IpcConstant::define_type`, so its type must be a `specta::Type` |

Inner indentation of a doc line survives; `#[doc(hidden)]` and other list-form doc attributes are skipped.

```rust
/// Frames VBAO accumulates over at most.
#[xrf_ipc::constant]
const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: u32 = 32;

fn main() {
  assert_eq!(RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION, 32);
}
```

## What it refuses

Each refusal is a compile error on the offending tokens:

- Any argument: `#[constant]` takes none.
- Anything but a non-generic `const` item: a `static`, a function, a generic `const`.
- A doc comment it cannot read as `///` lines: a block comment spanning lines, or a computed `#[doc = ...]`.
- An associated constant. Rust allows no module in an `impl` or a trait, so the error is Rust's own, on the item.

```rust,compile_fail
#[xrf_ipc::constant]
static RENDER_MAX_SHADOW_FILTER: u32 = 3;
```

## Checks

Run `cargo test --locked -p xrf-ipc-derive`. The trybuild cases are in `tests/ui`: `pass/` compiles and runs, `fail/`
holds each refused form with the diagnostic it must produce in its `.stderr`. The expansion is `ConstantAttribute`, in
[`constant_attribute.rs`](src/constant_attribute.rs).
