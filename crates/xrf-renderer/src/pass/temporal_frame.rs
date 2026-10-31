use glam::{Mat4, Vec2};

/// What the temporal antialiasing resolves one frame by: this frame's view projection, the last frame's and its view,
/// the texel's jitter, whether the history holds a frame, and how much brighter the history reads than it was drawn.
#[derive(Clone, Copy, Debug)]
pub struct TemporalFrame {
  pub current: Mat4,
  pub previous: Mat4,
  pub previous_view: Mat4,
  pub jitter: Vec2,
  pub is_history_valid: bool,
  pub history_scale: f32,
}
