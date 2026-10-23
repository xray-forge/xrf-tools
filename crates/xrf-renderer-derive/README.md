# xrf-renderer-derive

The three derives [xrf-renderer-core](../xrf-renderer-core/README.md) re-exports: `ShaderStruct`, `PassParameters` and
`ShaderPermutation`. Depend on the core and import them from there; their expansions name `::xrf_renderer_core` and
implement its traits, so they do not compile without it. Each derive takes a struct with named fields and reports what
it refuses as a compile error on the offending tokens.

## `ShaderStruct`

Declares a struct a shader and Rust share once, in Rust. The derive lays the struct out by WGSL's host-shareable rules
in constants, from each member's `ShaderType`, and asserts at compile time that Rust laid it out the same: a member
WGSL places elsewhere, or a size WGSL rounds differently, fails the build naming the member. Members named with a
leading underscore are padding, written in Rust and left out of WGSL, which pads implicitly. The struct must be
`#[repr(C)]` and `bytemuck::Pod`, so a write stays a copy of its bytes, and needs one member that is not padding. WGSL
knows it by its Rust name unless `#[shader(name = "...")]` gives another, as a `LightingUniform` read as `Lighting`.
A generic or tuple struct is refused.

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

Where Rust leaves padding of its own, `bytemuck::Pod` already refuses the struct. Where Rust packs tighter than WGSL,
the derive does: Rust puts the second `Vec3` twelve bytes in, WGSL sixteen, so the padding has to be written out:

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

Declares one bind group of a pass as a struct: a binding per field, at the group `#[parameters(group = G)]` names
(required, once). Each field is numbered one past the field before, starting at 0, unless `#[binding(N)]` numbers it,
so passes drawing with one module can keep their bindings apart; an index taken twice is refused. From the struct come
the bind group's layout, the WGSL declaring its bindings (named as the fields are), the graph accesses the pass makes,
and what each binding binds this frame. The cached layout is keyed by the struct's module path and name
(`LAYOUT_KEY`), so type and const parameters are refused; a lifetime parameter is allowed.

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

- A uniform is a value pushed to the graph runtime's upload ring this frame and read at its dynamic offset. A storage
  value is one `T` read from the buffer's head. The `Mut` storage forms are read and written (`read_write`). A storage
  array whose elements WGSL places further apart than Rust, as `Vec3`'s, fails the build.
- `#[texture(dimension, sample)]`: the dimension is `d2`, `d2_array`, `cube`, `cube_array` or `d3`, and the texture
  samples `float`, `unfilterable`, `uint`, `sint` or `depth`; a three-dimensional depth texture is refused. On an array
  field it is a binding array, every element sampled, and the struct's `ENABLES` names `wgpu_binding_array`, which
  `ShaderBindings` writes as an `enable` line above the bindings.
- `#[storage_texture(dimension, format, access)]`: `d2`, `d2_array` or `d3`; `rgba8unorm`, `rgba16float`,
  `rgba32float`, `rg32float`, `r32float`, `r32uint`, `r32sint` or `rgba32uint`; and `read`, `write` or `read_write`.
- `#[sampler(kind)]`: `filtering`, `non_filtering` or `comparison`.
- An acceleration structure is a top-level one rays are traced against; `ENABLES` then names `wgpu_ray_query`.
- `#[uniform]`, `#[storage]` and `#[acceleration_structure]` take no arguments. A word a kind does not take, or a second
  word of one class (two dimensions, say), is refused rather than dropped.

A binding is visible to every stage, except a writable one or an acceleration structure, which a vertex shader may not
hold. A field's name is its WGSL name, so it must not be one of WGSL's reserved words (`target`, `filter`, …); the
derive does not check, and the naga check in a pass's tests is what catches one.

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

Declares a shader's permutation domain: each field a WGSL `override` constant of its name in upper case (a raw
identifier's `r#` dropped), as WGSL constants are named, its type from `ShaderOverride` (`bool`, `u32`, `i32`, `f32`,
or an implementation of one's own), set from the field's value when a pipeline is made. It is the only way a pipeline
is given override constants. A generic struct is refused.

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

## Checks

Run `cargo test --locked -p xrf-renderer-derive`. The trybuild cases in `tests/ui` are refused forms, each with the
diagnostic it must produce in its `.stderr`: a misaligned member, a tuple struct, a storage array of mismatched stride,
a taken binding index, a repeated group, a generic parameter struct, a contradicting texture word, a three-dimensional
depth texture, and an argument to `#[uniform]`. The expansions are in [`shader_struct.rs`](src/shader_struct.rs),
[`pass_parameters.rs`](src/pass_parameters.rs) and [`shader_permutation.rs`](src/shader_permutation.rs).
