use std::time::Duration;

/// What a view's frame gives DLSS to upscale.
pub struct DlssInputs<'a> {
  /// The frame as drawn, at the render size.
  pub color: &'a wgpu::TextureView,
  /// Its depth, reversed: one at the near plane.
  pub depth: &'a wgpu::TextureView,
  /// Each pixel's motion at the render size, which `motion_scale` turns into render pixels from where it stands now
  /// back to where it stood the frame before.
  pub motion: &'a wgpu::TextureView,
  pub motion_scale: [f32; 2],
  /// How far each pixel is upscaled from this frame rather than the history, from nothing to one, where something
  /// drawn over the surfaces changed it (rain, particles, water), whose motion is the surface's behind it: none to
  /// leave every pixel to the network.
  pub bias_current_color: Option<&'a wgpu::TextureView>,
  /// Where this frame's samples sit within their pixels, in render pixels.
  pub jitter: [f32; 2],
  /// The render size, within the upscaler's range.
  pub render: (u32, u32),
  /// The texture DLSS writes the upscaled frame into, at the output's size, bound as storage.
  pub output: &'a wgpu::TextureView,
  /// Whether the frames before are forgotten: the first frame, or one after a cut.
  pub is_reset: bool,
  /// Time since the frame before, which DLSS weighs its history by.
  pub frame_time: Duration,
}

impl DlssInputs<'_> {
  /// The views NGX reads.
  #[cfg(feature = "sdk")]
  pub(crate) fn get_read(&self) -> Vec<&wgpu::TextureView> {
    let mut views: Vec<&wgpu::TextureView> = vec![self.color, self.depth, self.motion];

    views.extend(self.bias_current_color);

    views
  }
}
