# xrf-renderer-derive

The derives `xrf-renderer-core` re-exports; depend on the core and name them from there.

## `ShaderStruct`

Declares a struct a shader and Rust share once, in Rust. The derive lays the struct out by WGSL's host-shareable rules
in constants, from each member's `ShaderType`, and asserts at compile time that Rust laid it out the same: a member
WGSL places elsewhere, or a size WGSL rounds differently, fails the build with the member named. Members named with a
leading underscore are padding, written in Rust and left out of WGSL, which pads implicitly. The struct must be
`#[repr(C)]` and `bytemuck::Pod`, so a write stays a copy of its bytes. WGSL knows it by its Rust name unless
`#[shader(name = "...")]` gives another, as a `LightingUniform` read as `Lighting`.

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
the derive does: Rust puts the second `Vec3` twelve bytes in, WGSL sixteen, so the padding is written out:

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

Declares one bind group of a pass as a struct: a binding per field, at the group `#[parameters(group = G)]` names,
each numbered one past the field before unless `#[binding(N)]` numbers it (an index taken twice is refused), so passes
drawing with one module can keep their bindings apart. From it come the bind group's layout, the WGSL that declares its bindings (named as the fields are), the graph
accesses the pass makes, and what each binding binds this frame. Field kinds:

- `#[uniform]` on a `UniformBinding<'_, T>`: a value pushed to the upload ring this frame, read at its dynamic offset;
  `var<uniform> name: T;`.
- `#[storage]` on a `StorageArray<T>` or `StorageArrayMut<T>`: a graph buffer read, or read and written, as
  `array<T>`.
- `#[texture(dimension, sample)]` on a `GraphTexture`: `d2`, `d2_array`, `cube`, `cube_array` or `d3`, sampled as
  `float`, `unfilterable`, `uint`, `sint` or `depth`. On a `[GraphTexture; N]` it is a binding array of `N`,
  `binding_array<T, N>`, every element sampled; the struct's `ENABLES` then names `wgpu_binding_array`, which
  `ShaderBindings` writes as an `enable` line above the bindings.
- `#[storage_texture(dimension, format, access)]` on a `GraphTexture`: `d2`, `d2_array` or `d3`, a format such as
  `rgba16float`, and `read`, `write` or `read_write`.
- `#[sampler(kind)]` on a `&wgpu::Sampler`: `filtering`, `non_filtering` or `comparison`.

A binding is visible to every stage, except a writable one, which a vertex shader may not hold. A field's name is its
WGSL name, so it must not be one of WGSL's reserved words (`target`, `filter`, …); the naga check in a pass's tests is
what catches one.

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

Declares a shader's permutation domain: each field a WGSL `override` constant of its name, its type from
`ShaderOverride` (`bool`, `u32`, `i32`, `f32`, or an implementation of one's own), set from the field's value when a
pipeline is made.

```rust
use xrf_renderer_core::ShaderPermutation;

#[derive(ShaderPermutation)]
struct Shadowing {
  is_shadow: bool,
  cascade: u32,
}

let shadowing = Shadowing { is_shadow: true, cascade: 2 };

assert_eq!(shadowing.list_constants(), [("is_shadow", 1.0), ("cascade", 2.0)]);
assert_eq!(Shadowing::get_wgsl_overrides(), "override is_shadow: bool;
override cascade: u32;
");
```
