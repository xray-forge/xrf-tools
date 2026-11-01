/// What a frame is presented into, chosen by where it goes: a window's swapchain in the format the window picked, or the
/// texture a capture draws apart from its window. Every target is 8-bit and takes the display transform's sRGB values;
/// a wider one, an HDR display's, is issue 0224's.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DisplayTarget {
  pub format: wgpu::TextureFormat,
}

impl DisplayTarget {
  /// What a screenshot, a capture's shot or a burst's frame is drawn into, whatever its window draws into.
  pub const CAPTURE: Self = Self {
    format: wgpu::TextureFormat::Rgba8Unorm,
  };
}
