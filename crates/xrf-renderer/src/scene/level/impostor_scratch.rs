use crate::scene::level::impostor_atlas::ImpostorAtlas;

/// Where a level's blocks of the three maps go in the bake's block buffer: its texels a side, its blocks a side, the
/// blocks a row takes there (a copy's row alignment), and where each map's blocks start, in blocks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImpostorBlockLevel {
  pub size: u32,
  pub blocks: u32,
  pub row: u32,
  pub starts: [u32; 3],
}

impl ImpostorBlockLevel {
  /// Blocks a row of the block buffer is padded to, so a row's bytes are a copy's row alignment.
  const ROW_BLOCKS: u32 = (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64 / ImpostorAtlas::BLOCK_BYTES) as u32;

  /// Every level's place in the block buffer of an atlas `size` texels a side with `levels` mips, each level's maps
  /// one after another, and the blocks they take in all.
  pub fn list(size: u32, levels: u32) -> (Vec<Self>, u32) {
    let mut block_count: u32 = 0;
    let blocks: Vec<Self> = (0..levels)
      .map(|level| {
        let across: u32 = ImpostorAtlas::get_blocks(size, level);
        let row: u32 = across.next_multiple_of(Self::ROW_BLOCKS);
        let starts: [u32; 3] = std::array::from_fn(|map| block_count + map as u32 * row * across);

        block_count += 3 * row * across;

        Self {
          size: (size >> level).max(1),
          blocks: across,
          row,
          starts,
        }
      })
      .collect();

    (blocks, block_count)
  }
}

/// The uncompressed maps a bake packs its frames into and reduces level by level, before compressing each level into
/// a type's atlases (`ImpostorAtlas`'s layout, each `Rgba8Unorm`), kept a size; and where each level's blocks go.
pub struct ImpostorScratch {
  pub size: u32,
  /// Each level's view of each map.
  pub levels: Vec<[wgpu::TextureView; 3]>,
  pub blocks: Vec<ImpostorBlockLevel>,
  /// Blocks every level of every map takes in the block buffer.
  pub block_count: u32,
}

impl ImpostorScratch {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  pub fn new(device: &wgpu::Device, size: u32, levels: u32) -> Self {
    let textures: [wgpu::Texture; 3] =
      ["impostor colour bake", "impostor normal bake", "impostor material bake"].map(|label| {
        device.create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
          },
          mip_level_count: levels,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
      });
    let (blocks, block_count): (Vec<ImpostorBlockLevel>, u32) = ImpostorBlockLevel::list(size, levels);

    Self {
      size,
      levels: (0..levels)
        .map(|level| {
          std::array::from_fn(|map| {
            textures[map].create_view(&wgpu::TextureViewDescriptor {
              base_mip_level: level,
              mip_level_count: Some(1),
              ..Default::default()
            })
          })
        })
        .collect(),
      blocks,
      block_count,
    }
  }
}
