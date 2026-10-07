use glam::{Mat4, Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

/// What the temporal resolve reads besides the frame's targets, as WGSL's `Temporal`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Temporal")]
pub struct TemporalUniform {
  pub current: Mat4,
  pub previous: Mat4,
  pub previous_view: Mat4,
  pub params: Vec4,
}

impl TemporalUniform {
  /// The least share of this frame a pixel takes: its history is roughly this many frames' average, inverted.
  pub const CURRENT_WEIGHT: f32 = 0.1;

  pub fn new(current: Mat4, previous: Mat4, previous_view: Mat4, jitter: Vec2, is_history_valid: bool) -> Self {
    Self {
      current,
      previous,
      previous_view,
      params: Vec4::new(
        jitter.x,
        jitter.y,
        f32::from(u8::from(is_history_valid)),
        Self::CURRENT_WEIGHT,
      ),
    }
  }
}
