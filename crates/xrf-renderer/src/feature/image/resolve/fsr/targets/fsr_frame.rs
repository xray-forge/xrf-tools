use crate::frame::frame_texture::FrameTexture;

/// What FSR writes each frame for the next to read: at the drawn size the luma, how much each texel trusted the
/// history, and the four last lumas; at the viewport's size the history, its colour and each pixel's lock.
pub struct FsrFrame {
  pub luma: FrameTexture,
  pub accumulation: FrameTexture,
  pub luma_history: FrameTexture,
  pub history: FrameTexture,
}

impl FsrFrame {
  pub fn get_memory(&self) -> u64 {
    [&self.luma, &self.accumulation, &self.luma_history, &self.history]
      .iter()
      .map(|texture| texture.get_memory())
      .sum()
  }
}
