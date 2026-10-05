use glam::{Vec2, Vec3};
use xrf_renderer_core::ShaderStruct;

/// One corner of a particle's quad as `shaders/frame/particles.wgsl` reads it, `FVF::LIT` with its surface: the corner
/// in renderer space, its colour, its texture coordinate and the surface its effect draws with.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub struct ParticleVertex {
  pub position: Vec3,
  /// Red, green, blue and alpha bytes, red lowest, as `unpack4x8unorm` reads them.
  pub color: u32,
  pub uv: Vec2,
  /// Its effect's surface, by its index among the level's particle surfaces.
  pub surface: u32,
  pub _pad: u32,
}

impl ParticleVertex {
  /// Corners a quad writes, which `QuadIB` draws as two triangles.
  pub const CORNERS: u32 = 4;
}
