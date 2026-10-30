use crate::scene::static_scene::static_impostor_frames::StaticImpostorFrames;

/// One tree type's baked impostor: three block-compressed atlases of `StaticImpostorFrames::FRAMES` square frames each
/// way, `size` texels a side, with `levels` mips:
/// - colour, and coverage in alpha (BC3);
/// - the normal in the type's own space, octahedral, in red and green (BC5);
/// - the gloss, the hemisphere term and the material slice in colour, and the depth across the type's sphere in alpha,
///   one at its front and nought at its back (BC3).
pub struct ImpostorAtlas {
  pub size: u32,
  pub levels: u32,
  pub textures: [wgpu::Texture; 3],
  pub views: [wgpu::TextureView; 3],
}

impl ImpostorAtlas {
  pub const FORMATS: [wgpu::TextureFormat; 3] = [
    wgpu::TextureFormat::Bc3RgbaUnorm,
    wgpu::TextureFormat::Bc5RgUnorm,
    wgpu::TextureFormat::Bc3RgbaUnorm,
  ];

  /// Bytes a block of each takes: sixteen texels.
  pub const BLOCK_BYTES: u64 = 16;

  /// Labels of the three atlases, in their order.
  const LABELS: [&'static str; 3] = ["impostor colour", "impostor normal", "impostor material"];

  pub fn new(device: &wgpu::Device, size: u32, levels: u32) -> Self {
    let textures: [wgpu::Texture; 3] = std::array::from_fn(|map| {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(Self::LABELS[map]),
        size: wgpu::Extent3d {
          width: size,
          height: size,
          depth_or_array_layers: 1,
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::FORMATS[map],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      })
    });

    Self {
      size,
      levels,
      views: std::array::from_fn(|map| textures[map].create_view(&wgpu::TextureViewDescriptor::default())),
      textures,
    }
  }

  /// Writes every level of the three atlases from `blocks`, each level's maps one after another, a row of blocks
  /// straight after the last, as the cache keeps them.
  pub fn write(&self, queue: &wgpu::Queue, blocks: &[u8]) {
    let mut at: u64 = 0;

    for level in 0..self.levels {
      let across: u32 = Self::get_blocks(self.size, level);
      let bytes: u64 = u64::from(across).pow(2) * Self::BLOCK_BYTES;

      for texture in &self.textures {
        queue.write_texture(
          wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
          },
          &blocks[at as usize..(at + bytes) as usize],
          wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(across * Self::BLOCK_BYTES as u32),
            rows_per_image: Some(across),
          },
          wgpu::Extent3d {
            width: across * 4,
            height: across * 4,
            depth_or_array_layers: 1,
          },
        );
        at += bytes;
      }
    }
  }

  /// Texels a side of a frame of an atlas `size` texels a side.
  pub fn get_frame(size: u32) -> u32 {
    size / StaticImpostorFrames::FRAMES
  }

  /// Mips an atlas of `frame`-texel frames keeps: down to frames four texels across, below which each frame would be a
  /// block of its own blurring into its neighbours.
  pub fn get_levels(frame: u32) -> u32 {
    (frame / 4).max(1).ilog2() + 1
  }

  /// Blocks a side of a level of an atlas `size` texels a side.
  pub fn get_blocks(size: u32, level: u32) -> u32 {
    (size >> level).max(1).div_ceil(4)
  }

  /// Bytes the three atlases `size` texels a side with `levels` mips take.
  pub fn get_bytes(size: u32, levels: u32) -> u64 {
    (0..levels)
      .map(|level| u64::from(Self::get_blocks(size, level)).pow(2) * Self::BLOCK_BYTES)
      .sum::<u64>()
      * Self::FORMATS.len() as u64
  }

  /// Bytes its atlases take.
  pub fn get_memory(&self) -> u64 {
    Self::get_bytes(self.size, self.levels)
  }
}
