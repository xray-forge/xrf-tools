use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};

/// What the temporal resolve reads besides the frame's targets, as `frame/temporal.wgsl`'s `Temporal` lays it out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TemporalUniform {
  pub current: [f32; 16],
  pub previous: [f32; 16],
  pub previous_view: [f32; 16],
  pub params: [f32; 4],
}

impl TemporalUniform {
  /// The least share of this frame a pixel takes: its history is roughly this many frames' average, inverted.
  pub const CURRENT_WEIGHT: f32 = 0.1;

  pub fn new(current: Mat4, previous: Mat4, previous_view: Mat4, jitter: Vec2, is_history_valid: bool) -> Self {
    Self {
      current: current.to_cols_array(),
      previous: previous.to_cols_array(),
      previous_view: previous_view.to_cols_array(),
      params: [
        jitter.x,
        jitter.y,
        f32::from(u8::from(is_history_valid)),
        Self::CURRENT_WEIGHT,
      ],
    }
  }
}
