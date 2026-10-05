use glam::{Mat4, Vec2};
use xrf_renderer_core::ShaderStruct;

/// The view a depth pyramid was reduced through, as `shaders/static/cull.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Occlusion")]
pub struct StaticOcclusionUniform {
  pub view: Mat4,
  pub projection: Mat4,
  /// The pyramid's first level's size.
  pub size: Vec2,
  pub levels: u32,
  /// Whether the pyramid holds an earlier frame's depth drawn through this view.
  pub has_history: u32,
}
