use xrf_dlss::DlssUpscaler;

use crate::frame::frame_texture::FrameTexture;

/// DLSS's state: its upscaler, the frame it writes at the viewport's size, and whether the next frame forgets the ones
/// before.
pub struct DlssResolve {
  pub upscaler: DlssUpscaler,
  pub output: FrameTexture,
  pub is_reset: bool,
}

impl DlssResolve {
  /// What DLSS writes the upscaled frame as: colour of the scene's range, bound as storage.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
}
