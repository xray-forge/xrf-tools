use glam::{Mat4, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::pass::temporal_frame::TemporalFrame;

/// What the temporal resolve reads besides the frame's targets, as WGSL's `Temporal`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Temporal")]
pub struct TemporalUniform {
  pub current: Mat4,
  pub previous: Mat4,
  pub previous_view: Mat4,
  pub params: Vec4,
  /// What the history is multiplied by to stand at this frame's pre-exposure, in `x`.
  pub exposure: Vec4,
}

impl TemporalUniform {
  /// The least share of this frame a pixel takes: its history is roughly this many frames' average, inverted.
  pub const CURRENT_WEIGHT: f32 = 0.1;

  pub fn new(frame: TemporalFrame) -> Self {
    Self {
      current: frame.current,
      previous: frame.previous,
      previous_view: frame.previous_view,
      params: Vec4::new(
        frame.jitter.x,
        frame.jitter.y,
        f32::from(u8::from(frame.is_history_valid)),
        Self::CURRENT_WEIGHT,
      ),
      exposure: Vec4::new(frame.history_scale, 0.0, 0.0, 0.0),
    }
  }
}
