use crate::frame::view_targets::ViewTargets;

/// The surroundings cube's faces as last drawn: each face's G-buffer kept at the size it is drawn at, so a face whose
/// geometry still stands is lit again from it rather than culled and drawn again; and a spare, which a face drawn
/// again band by band is drawn into before it takes the face's place.
pub struct SurroundingsFaces {
  /// Texels a side each face is drawn at.
  pub drawn: u32,
  /// Every layer's albedo, normal, material and depth, in the order `SurroundingsFaces::FORMATS` names them.
  layers: [[wgpu::TextureView; 4]; Self::LAYERS],
  /// The layer each face keeps, and the spare.
  slots: [usize; 6],
  spare: usize,
}

impl SurroundingsFaces {
  /// What each face keeps: the G-buffer the surroundings' resolve reads.
  pub const FORMATS: [(&'static str, wgpu::TextureFormat); 4] = [
    ("surroundings albedo", ViewTargets::ALBEDO),
    ("surroundings normal", ViewTargets::NORMAL),
    ("surroundings material", ViewTargets::MATERIAL),
    ("surroundings depth", ViewTargets::DEPTH),
  ];
  /// Layers kept: a face each, and the spare.
  const LAYERS: usize = 7;

  pub fn new(device: &wgpu::Device, drawn: u32) -> Self {
    let textures: [wgpu::Texture; 4] = Self::FORMATS.map(|(label, format)| {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width: drawn,
          height: drawn,
          depth_or_array_layers: Self::LAYERS as u32,
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
      layers: std::array::from_fn(|layer| {
        std::array::from_fn(|target| {
          textures[target].create_view(&wgpu::TextureViewDescriptor {
            label: Some(Self::FORMATS[target].0),
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: layer as u32,
            array_layer_count: Some(1),
            ..Default::default()
          })
        })
      }),
      slots: std::array::from_fn(|face| face),
      spare: 6,
    }
  }

  /// The G-buffer a face keeps.
  pub fn get_face(&self, face: u32) -> &[wgpu::TextureView; 4] {
    &self.layers[self.slots[face as usize]]
  }

  /// The G-buffer a face drawn again band by band is drawn into.
  pub fn get_spare(&self) -> &[wgpu::TextureView; 4] {
    &self.layers[self.spare]
  }

  /// Lets the spare, drawn whole, take a face's place, the face's old layer the spare.
  pub fn swap(&mut self, face: u32) {
    std::mem::swap(&mut self.slots[face as usize], &mut self.spare);
  }

  /// Bytes every layer takes.
  pub fn get_memory(&self) -> u64 {
    let texel: u64 = Self::FORMATS
      .iter()
      .map(|(_, format)| {
        let bytes: Option<u32> = format
          .block_copy_size(None)
          .or_else(|| format.block_copy_size(Some(wgpu::TextureAspect::DepthOnly)));

        u64::from(bytes.unwrap_or(4))
      })
      .sum();

    u64::from(self.drawn).pow(2) * Self::LAYERS as u64 * texel
  }
}
