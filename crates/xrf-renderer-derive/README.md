# xrf-renderer-derive

The derives `xrf-renderer-core` re-exports; depend on the core and name them from there.

## `ShaderStruct`

Declares a struct a shader and Rust share once, in Rust. The derive lays the struct out by WGSL's host-shareable rules
in constants, from each member's `ShaderType`, and asserts at compile time that Rust laid it out the same: a member
WGSL places elsewhere, or a size WGSL rounds differently, fails the build with the member named. Members named with a
leading underscore are padding, written in Rust and left out of WGSL, which pads implicitly. The struct must be
`#[repr(C)]` and `bytemuck::Pod`, so a write stays a copy of its bytes.

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
