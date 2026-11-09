# xrf-renderer-derive

The renderer's derive macros.

| Derive                | Declares                                       | Use from            |
| --------------------- | ---------------------------------------------- | ------------------- |
| `ShaderStruct`        | A struct shared between WGSL and Rust          | `xrf_renderer_core` |
| `PassParameters`      | One bind group of a pass                       | `xrf_renderer_core` |
| `ShaderPermutation`   | A shader's `override` constants                | `xrf_renderer_core` |
| `RenderSettingSchema` | A renderer setting type's bounds, for the page | `xrf-renderer` only |

Depend on [xrf-renderer-core](../xrf-renderer-core/README.md) and import the first three from there: their expansions
name `::xrf_renderer_core` and implement its traits, so they do not compile without it.

Each derive takes a struct with named fields and reports what it refuses as a compile error on the offending tokens.

## `ShaderStruct`

Declares, once in Rust, a struct that a shader and Rust share.

- The derive computes WGSL's host-shareable layout from each member's `ShaderType`, and asserts at compile time that
  Rust's layout matches. A member WGSL places elsewhere, or a size WGSL rounds differently, fails the build naming the
  member.
- Members whose names start with an underscore are padding: written in Rust, left out of WGSL (which pads implicitly).
- The struct must be `#[repr(C)]` and `bytemuck::Pod`, so a write is a copy of its bytes, with at least one member that
  is not padding.
- WGSL knows it by its Rust name unless `#[shader(name = "...")]` gives another (a `LightingUniform` read as
  `Lighting`).
- A generic or tuple struct is refused.

```rust
use glam::{Vec3, Vec4};
use xrf_renderer_core::{ShaderStruct, ShaderType};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
struct Light {
  position: Vec3,
  radius: f32,
  color: Vec4,
}

assert_eq!(Light::SIZE, 32);
assert_eq!(
  Light::get_wgsl_declaration(),
  "struct Light {\n  position: vec3<f32>,\n  radius: f32,\n  color: vec4<f32>,\n}\n"
);
```

Where Rust would add padding of its own, `bytemuck::Pod` already refuses the struct. Where Rust packs tighter than WGSL,
the derive refuses it. Here Rust puts the second `Vec3` twelve bytes in and WGSL sixteen, so the padding must be
written out:

```rust,compile_fail
use glam::Vec3;
use xrf_renderer_core::ShaderStruct;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
struct Box3 {
  min: Vec3,
  // Missing: _pad: f32,
  max: Vec3,
}
```

## `PassParameters`

Declares one bind group of a pass as a struct, one binding per field.

- `#[parameters(group = G)]` names the group; it is required, once.
- Fields are numbered from 0, each one past the previous, unless `#[binding(N)]` numbers it, so passes sharing a module
  can keep their bindings apart. An index used twice is refused.
- The struct yields the bind group's layout, the WGSL declaring its bindings (named after the fields), the graph
  accesses the pass makes, and what each binding binds this frame.
- The cached layout is keyed by the struct's module path and name (`LAYOUT_KEY`), so type and const parameters are
  refused. A lifetime parameter is allowed.

Each field carries exactly one kind attribute:

| Attribute                   | Field type                              | WGSL declares                       |
| --------------------------- | --------------------------------------- | ----------------------------------- |
| `#[uniform]`                | `UniformBinding<T>`                     | `var<uniform> name: T`              |
| `#[storage]`                | `StorageArray<T>`, `StorageArrayMut<T>` | `var<storage, read> name: array<T>` |
| `#[storage]`                | `StorageValue<T>`, `StorageValueMut<T>` | `var<storage, read> name: T`        |
| `#[texture(..)]`            | `GraphTexture`                          | `texture_2d<f32>` and its kin       |
| `#[texture(..)]`            | `[GraphTexture; N]`                     | `binding_array<T, N>`               |
| `#[storage_texture(..)]`    | `GraphTexture`                          | `texture_storage_2d<F, A>` and kin  |
| `#[sampler(..)]`            | `&wgpu::Sampler`                        | `sampler`, `sampler_comparison`     |
| `#[acceleration_structure]` | `AccelerationStructureBinding<'_>`      | `acceleration_structure`            |

- A uniform is a value pushed to the graph runtime's upload ring this frame and read at its dynamic offset.
- A storage value is one `T` read from the buffer's start. The `Mut` storage forms are `read_write`. A storage array
  whose elements WGSL spaces further apart than Rust (such as `Vec3`) fails the build.
- `#[texture(dimension, sample)]`: dimension `d2`, `d2_array`, `cube`, `cube_array` or `d3`; sample type `float`,
  `unfilterable`, `uint`, `sint` or `depth`. A 3D depth texture is refused. On an array field it is a binding array,
  and the struct's `ENABLES` names `wgpu_binding_array`, which `ShaderBindings` writes as an `enable` line.
- `#[storage_texture(dimension, format, access)]`: `d2`, `d2_array` or `d3`; `rgba8unorm`, `rgba16float`,
  `rgba32float`, `rg32float`, `r32float`, `r32uint`, `r32sint` or `rgba32uint`; and `read`, `write` or `read_write`.
- `#[sampler(kind)]`: `filtering`, `non_filtering` or `comparison`.
- An acceleration structure is a top-level one rays are traced against; `ENABLES` then names `wgpu_ray_query`.
- `#[uniform]`, `#[storage]` and `#[acceleration_structure]` take no arguments. A word a kind does not take, or a second
  word of one class (two dimensions, say), is refused rather than dropped.

- A binding is visible to every stage, except a writable one or an acceleration structure, which a vertex shader may
  not hold.
- A field's name is its WGSL name, so it must not be a WGSL reserved word (`target`, `filter`, …). The derive does not
  check this; the naga check in a pass's tests catches it.

```rust
use xrf_renderer_core::{GraphTexture, PassParameters};

#[derive(PassParameters)]
#[parameters(group = 1)]
struct Shade {
  #[texture(d2, float)]
  source: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  destination: GraphTexture,
}

assert_eq!(
  Shade::get_wgsl_bindings(),
  "@group(1) @binding(0) var source: texture_2d<f32>;\n\
   @group(1) @binding(1) var destination: texture_storage_2d<rgba16float, write>;\n"
);
```

## `ShaderPermutation`

Declares a shader's permutations. It is the only way a pipeline is given override constants.

- Each field is a WGSL `override` constant named after the field in upper case (a raw identifier's `r#` dropped).
- Its type comes from `ShaderOverride`: `bool`, `u32`, `i32`, `f32`, or your own implementation.
- It is set from the field's value when a pipeline is made.
- A generic struct is refused.

```rust
use xrf_renderer_core::ShaderPermutation;

#[derive(ShaderPermutation)]
struct Shadowing {
  is_shadow: bool,
  cascade: u32,
}

let shadowing = Shadowing { is_shadow: true, cascade: 2 };

assert_eq!(shadowing.list_constants(), [("IS_SHADOW", 1.0), ("CASCADE", 2.0)]);
assert_eq!(Shadowing::get_wgsl_overrides(), "override IS_SHADOW: bool;
override CASCADE: u32;
");
```

## `RenderSettingSchema`

Describes one of the renderer contract's setting types for the page, which reads every setting's bounds from it
(`render_feature_schema`, exported to the bindings as `RENDER_FEATURE_SCHEMA`). It expands to `xrf-renderer`'s own
`crate::contract::settings::schema`, so it is derived only there.

A struct describes each field under its serialized name (as written, or under `rename_all = "camelCase"` or a field's
`rename`), according to its `#[setting(..)]`:

- None: what its type describes, a `bool` a flag, another setting type its group or choice.
- `min = .., max = ..`, optionally `integer`, `recommended = [.., ..]` and `wasted_above = ..`: a number between
  bounds, whole where `integer` says so or its type is; each value any expression that casts to `f32`, a contract
  constant included.
- `widths, min = .., max = .., most = ..`: a run of widths, as the sun's cascades are.
- `skip`: not a setting, as the look's, the weather's or the view's own values are not.

A unit enum is a choice of its variants as serde writes them (a variant's `rename`, or the enum's `rename_all`). A
generic type, a tuple struct and an enum with data are refused.

## Checks

Run `cargo test --locked -p xrf-renderer-derive`.

The trybuild cases in `tests/ui` are refused forms, each with the diagnostic it must produce in its `.stderr`:

- a misaligned member, or a tuple struct;
- a storage array with mismatched stride;
- a binding index used twice, or a repeated group;
- a generic parameter struct;
- a contradicting texture word, or a 3D depth texture;
- an argument to `#[uniform]`.

The expansions are in [`shader_struct.rs`](src/shader_struct.rs),
[`pass_parameters.rs`](src/pass_parameters.rs), [`shader_permutation.rs`](src/shader_permutation.rs) and
[`setting_schema.rs`](src/setting_schema.rs); the setting schema's expansion is tested in `xrf-renderer`, whose
contract it names.
