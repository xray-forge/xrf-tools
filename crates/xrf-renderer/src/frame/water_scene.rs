use crate::frame::view_targets::ViewTargets;

/// The scene as it stood before the water, copied from the view's scene each frame for the enhanced water to refract.
pub struct WaterScene {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  /// The targets' epoch it was sized for, made again past it.
  pub epoch: u64,
}

impl WaterScene {
  pub fn new(device: &wgpu::Device, targets: &ViewTargets, epoch: u64) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("water scene"),
      size: targets.scene_texture.size(),
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: ViewTargets::SCENE,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });

    Self {
      view: texture.create_view(&Default::default()),
      texture,
      epoch,
    }
  }

  /// Copies the scene drawn so far, before the water draws over it.
  pub fn copy(&self, encoder: &mut wgpu::CommandEncoder, targets: &ViewTargets) {
    encoder.copy_texture_to_texture(
      targets.scene_texture.as_image_copy(),
      self.texture.as_image_copy(),
      self.texture.size(),
    );
  }
}
