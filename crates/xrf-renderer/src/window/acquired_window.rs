use std::time::Duration;

use crate::contract::render_rect::RenderRect;
use crate::contract::render_viewport_id::RenderViewportId;

/// A window whose image this frame draws into: the image and its format, each viewport drawn into it with its whole
/// rectangle and the part the window shows, what the page is cleared to around them, whether its wash is painted, and
/// how long the image took to acquire.
pub struct AcquiredWindow {
  pub key: u64,
  pub image: wgpu::SurfaceTexture,
  pub format: wgpu::TextureFormat,
  pub drawn: Vec<(RenderViewportId, RenderRect, RenderRect)>,
  pub clear: wgpu::Color,
  pub is_washed: bool,
  pub acquire: Duration,
}
