use crate::frame::view_targets::ViewTargets;

/// The enhanced water's reflection: two histories at the drawn size, one written each frame over the other it reads,
/// and two at half the size its blur runs through, across then down.
pub struct WaterReflection {
  pub histories: [wgpu::TextureView; 2],
  pub blurred: [wgpu::TextureView; 2],
  /// The history this frame writes.
  pub index: usize,
  /// Whether the history this frame reads holds a frame.
  pub is_valid: bool,
  /// The targets' epoch it was sized for, made again past it.
  pub epoch: u64,
}

impl WaterReflection {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  pub fn new(device: &wgpu::Device, targets: &ViewTargets, epoch: u64) -> Self {
    let create = |label: &str, (width, height): (u32, u32)| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };
    let full: (u32, u32) = (targets.width, targets.height);
    let half: (u32, u32) = (targets.width.div_ceil(2), targets.height.div_ceil(2));

    Self {
      histories: [create("water reflection 0", full), create("water reflection 1", full)],
      blurred: [
        create("water reflection across", half),
        create("water reflection down", half),
      ],
      index: 0,
      is_valid: false,
      epoch,
    }
  }

  /// Makes the history just written the one the next frame reads.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
  }
}
