use crate::frame::targets::frame_texture::FrameTexture;

/// What FSR writes each frame for the next to read: at the drawn size the luma, how much each texel trusted the
/// history, and the four last lumas; at the viewport's size the history, its colour and each pixel's lock.
pub struct FsrFrame {
  pub luma: FrameTexture,
  pub accumulation: FrameTexture,
  pub luma_history: FrameTexture,
  pub history: FrameTexture,
}
