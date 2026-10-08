use crate::frame::frame_texture::FrameTexture;

/// What FSR 2 writes each frame for the next to read: at the drawn size the nearest depth, the motion found there and
/// the luma the locks read; at the viewport's size the resolved colour with its temporal reactivity, the locks and the
/// four last lumas.
pub struct FsrFrame {
  pub dilated_depth: FrameTexture,
  pub dilated_motion: FrameTexture,
  pub lock_luma: FrameTexture,
  pub history: FrameTexture,
  pub lock_status: FrameTexture,
  pub luma_history: FrameTexture,
}

impl FsrFrame {
  pub fn get_memory(&self) -> u64 {
    [
      &self.dilated_depth,
      &self.dilated_motion,
      &self.lock_luma,
      &self.history,
      &self.lock_status,
      &self.luma_history,
    ]
    .iter()
    .map(|texture| texture.get_memory())
    .sum()
  }
}
