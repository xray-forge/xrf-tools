use crate::frame::sky_prefilter_level::SkyPrefilterLevel;

/// The sky as drawn, prefiltered by roughness into a cube's mips: the first mip the sky itself, each after it the sky
/// blurred by the lobe of a rougher surface, so a reflection that meets nothing reads the cube at the mip its
/// roughness names. Filled again every frame, as the keyframes blend.
pub struct SkyPrefilter {
  pub texture: wgpu::Texture,
  /// The whole cube, every mip, which the sky group binds.
  pub cube: wgpu::TextureView,
  /// Each mip as the pass fills it, the first first.
  pub levels: Vec<SkyPrefilterLevel>,
}

impl SkyPrefilter {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Texels a side of the first mip.
  pub const SIZE: u32 = 128;
  /// Mips, the first the sky as drawn; the last at roughness one.
  pub const LEVELS: u32 = 6;

  pub fn new(device: &wgpu::Device) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("sky prefiltered"),
      size: wgpu::Extent3d {
        width: Self::SIZE,
        height: Self::SIZE,
        depth_or_array_layers: 6,
      },
      mip_level_count: Self::LEVELS,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::FORMAT,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let cube: wgpu::TextureView = texture.create_view(&wgpu::TextureViewDescriptor {
      label: Some("sky prefiltered"),
      dimension: Some(wgpu::TextureViewDimension::Cube),
      ..Default::default()
    });
    let levels: Vec<SkyPrefilterLevel> = (0..Self::LEVELS)
      .map(|level| SkyPrefilterLevel {
        size: Self::SIZE >> level,
        source: (level > 0).then(|| {
          texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("sky prefiltered before"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            base_mip_level: 0,
            mip_level_count: Some(level),
            ..Default::default()
          })
        }),
        target: texture.create_view(&wgpu::TextureViewDescriptor {
          label: Some("sky prefiltered level"),
          dimension: Some(wgpu::TextureViewDimension::D2Array),
          base_mip_level: level,
          mip_level_count: Some(1),
          ..Default::default()
        }),
      })
      .collect();

    Self { texture, cube, levels }
  }
}
