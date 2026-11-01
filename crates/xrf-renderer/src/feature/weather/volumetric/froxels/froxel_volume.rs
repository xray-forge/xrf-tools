use crate::frame::history::History;

/// The enhanced volumetric lighting's froxels, a grid along the view: what each scatters and how thick it is, kept over
/// frames in a history, and what is scattered towards the camera up to each, integrated from them.
pub struct FroxelVolume {
  pub histories: History<wgpu::TextureView>,
  pub integrated: wgpu::TextureView,
}

impl FroxelVolume {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Froxels across the screen, down it and along the view.
  pub const SIZE: [u32; 3] = [160, 90, 64];

  pub fn new(device: &wgpu::Device) -> Self {
    let create = |label: &str| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: Self::SIZE[0],
            height: Self::SIZE[1],
            depth_or_array_layers: Self::SIZE[2],
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D3,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };

    Self {
      histories: History::new(|side| create(["froxel history 0", "froxel history 1"][side])),
      integrated: create("froxels integrated"),
    }
  }

  /// Bytes its three grids take: two histories and the integrated one, at half floats.
  pub fn get_memory(&self) -> u64 {
    let [width, height, depth] = Self::SIZE.map(u64::from);

    3 * width * height * depth * 8
  }
}
