use crate::frame::targets::view_targets::ViewTargets;

/// The surroundings cube's faces' G-buffers as last drawn, so a face whose geometry still stands is lit again rather
/// than drawn again, and a spare a face redrawn band by band is drawn into before it takes the face's place.
///
/// Each target is one array, a layer a face or the spare, imported into the frame graph whole: a draw attaches its
/// layer and the resolve reads it by index. A one-layer view imported alone would be declared as the array behind it,
/// so a draw into it would attach the first layer whatever layer the view shows.
pub(crate) struct SurroundingsFaces {
  /// Texels a side each face is drawn at.
  pub drawn: u32,
  /// The arrays in the order `SurroundingsFaces::FORMATS` names them, viewed whole.
  arrays: [wgpu::TextureView; 4],
  /// The layer each face keeps, and the spare.
  slots: [u32; 6],
  spare: u32,
}

impl SurroundingsFaces {
  /// What each face keeps: the G-buffer the surroundings' resolve reads.
  pub(crate) const FORMATS: [(&'static str, wgpu::TextureFormat); 4] = [
    ("surroundings albedo", ViewTargets::ALBEDO),
    ("surroundings normal", ViewTargets::NORMAL),
    ("surroundings material", ViewTargets::MATERIAL),
    ("surroundings depth", ViewTargets::DEPTH),
  ];
  /// Layers kept: a face each, and the spare.
  const LAYERS: u32 = 7;

  pub(crate) fn new(device: &wgpu::Device, drawn: u32) -> Self {
    Self {
      drawn,
      arrays: Self::FORMATS.map(|(label, format)| {
        device
          .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
              width: drawn,
              height: drawn,
              depth_or_array_layers: Self::LAYERS,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          })
          .create_view(&wgpu::TextureViewDescriptor {
            label: Some(label),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
          })
      }),
      slots: std::array::from_fn(|face| face as u32),
      spare: 6,
    }
  }

  /// The albedo, normal, material and depth arrays, viewed whole.
  pub(crate) fn get_arrays(&self) -> &[wgpu::TextureView; 4] {
    &self.arrays
  }

  /// The layer of the arrays a face keeps its G-buffer in.
  pub(crate) fn get_face_layer(&self, face: u32) -> u32 {
    self.slots[face as usize]
  }

  /// The layer a face drawn again band by band is drawn into.
  pub(crate) fn get_spare_layer(&self) -> u32 {
    self.spare
  }

  /// Lets the spare, once drawn whole, take a face's place, the face's old layer becoming the spare.
  pub(crate) fn swap(&mut self, face: u32) {
    std::mem::swap(&mut self.slots[face as usize], &mut self.spare);
  }
}
