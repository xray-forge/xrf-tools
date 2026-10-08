use crate::frame::view_targets::ViewTargets;

/// The surroundings cube's faces as last drawn: each face's G-buffer kept at the size it is drawn at, so a face whose
/// geometry still stands is lit again from it rather than culled and drawn again.
pub struct SurroundingsFaces {
  /// Texels a side each face is drawn at.
  pub drawn: u32,
  /// Each face's albedo, normal, material and depth, in the order `SurroundingsFaces::FORMATS` names them.
  pub layers: [[wgpu::TextureView; 4]; 6],
}

impl SurroundingsFaces {
  /// What each face keeps: the G-buffer the surroundings' resolve reads.
  pub const FORMATS: [(&'static str, wgpu::TextureFormat); 4] = [
    ("surroundings albedo", ViewTargets::ALBEDO),
    ("surroundings normal", ViewTargets::NORMAL),
    ("surroundings material", ViewTargets::MATERIAL),
    ("surroundings depth", ViewTargets::DEPTH),
  ];

  pub fn new(device: &wgpu::Device, drawn: u32) -> Self {
    let textures: [wgpu::Texture; 4] = Self::FORMATS.map(|(label, format)| {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width: drawn,
          height: drawn,
          depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
    });

    Self {
      drawn,
      layers: std::array::from_fn(|face| {
        std::array::from_fn(|target| {
          textures[target].create_view(&wgpu::TextureViewDescriptor {
            label: Some(Self::FORMATS[target].0),
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: face as u32,
            array_layer_count: Some(1),
            ..Default::default()
          })
        })
      }),
    }
  }
}
