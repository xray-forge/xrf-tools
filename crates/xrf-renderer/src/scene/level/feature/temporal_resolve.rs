use glam::Mat4;

use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::pass::temporal_uniform::TemporalUniform;

/// TAA's state: its colour and distance history at the viewport's size, made for the drawn size `render`, the last
/// resolved frame's view projection without its jitter and its view, and what it moves its history by this frame.
pub struct TemporalResolve {
  pub history: History<FrameTexture>,
  pub render: (u32, u32),
  pub previous: Option<(Mat4, Mat4)>,
  pub uniform: TemporalUniform,
}
