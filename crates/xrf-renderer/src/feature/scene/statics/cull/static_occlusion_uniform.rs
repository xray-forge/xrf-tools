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

impl StaticOcclusionUniform {
  /// The same, holding depth drawn through its view where `is_held`.
  pub fn with_history(self, is_held: bool) -> Self {
    Self {
      has_history: u32::from(is_held),
      ..self
    }
  }
}
