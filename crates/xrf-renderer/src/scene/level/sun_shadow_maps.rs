use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;

/// The sun's cascades' maps: one layer a cascade there can be, each a square of reversed depth seen from the sun. The
/// maps the sun samples, or the static layers, which hold what stands still and are copied into them.
pub struct SunShadowMaps {
  pub resolution: u32,
  /// Every layer, as the sun samples them.
  pub view: wgpu::TextureView,
  /// Each layer alone, as its cascade draws into it.
  pub layers: Vec<wgpu::TextureView>,
}

impl SunShadowMaps {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

  /// The maps the sun samples, which a static layer is copied into.
  pub fn new(device: &wgpu::Device, resolution: u32) -> Self {
    Self::create(
      device,
      (resolution, RENDER_MAX_SHADOW_CASCADES as u32),
      "sun shadow maps",
      wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    )
  }

  /// `layers` static layers, copied into the maps.
  pub fn new_static(device: &wgpu::Device, resolution: u32, layers: u32) -> Self {
    Self::create(
      device,
      (resolution, layers),
      "sun static shadow maps",
      wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    )
  }

  /// `layers` layers the trees are drawn into over a static layer's copy, kept between their draws and copied into the
  /// maps.
  pub fn new_kept(device: &wgpu::Device, resolution: u32, layers: u32) -> Self {
    Self::create(
      device,
      (resolution, layers),
      "sun kept shadow maps",
      wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
    )
  }

  fn create(
    device: &wgpu::Device,
    (resolution, layers): (u32, u32),
    label: &'static str,
    usage: wgpu::TextureUsages,
  ) -> Self {
    let resolution: u32 = resolution.clamp(1, device.limits().max_texture_dimension_2d);
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width: resolution,
        height: resolution,
        depth_or_array_layers: layers,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::FORMAT,
      usage,
      view_formats: &[],
    });
    let views: Vec<wgpu::TextureView> = (0..layers)
      .map(|layer| {
        texture.create_view(&wgpu::TextureViewDescriptor {
          dimension: Some(wgpu::TextureViewDimension::D2),
          base_array_layer: layer,
          array_layer_count: Some(1),
          ..Default::default()
        })
      })
      .collect();

    Self {
      resolution,
      view: texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
      }),
      layers: views,
    }
  }
}
