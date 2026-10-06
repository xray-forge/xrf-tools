use glam::Vec3;
use xrf_renderer_core::ShaderStruct;

// Rust packs the second vec3 twelve bytes in; WGSL aligns it to sixteen.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
struct Box3 {
  min: Vec3,
  max: Vec3,
}

fn main() {}
