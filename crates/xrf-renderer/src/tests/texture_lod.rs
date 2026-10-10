use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::texture_cache::get_dropped_levels;

/// A block compressed texture of `width` by `height` with every level down to four texels.
fn create_texture(width: u32, height: u32) -> DecodedTexture {
  let levels: Vec<Vec<u8>> = (0..)
    .map(|level: u32| (width >> level, height >> level))
    .take_while(|&(width, height)| width >= 4 && height >= 4)
    .map(|(width, height)| vec![0; (width * height) as usize])
    .collect();

  DecodedTexture {
    format: wgpu::TextureFormat::Bc3RgbaUnorm,
    width,
    height,
    layers: 1,
    is_volume: false,
    levels,
    layout: String::from("DXT5"),
    is_expanded: false,
  }
}

#[test]
fn texture_lod_drops_the_engines_levels_and_a_listed_texture_a_step_sooner() {
  let table: Vec<(u32, u32)> = (0..=4)
    .map(|lod| (get_dropped_levels(lod, false), get_dropped_levels(lod, true)))
    .collect();

  assert_eq!(table, vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)]);
}

#[test]
fn texture_lod_cuts_the_largest_levels_and_halves_the_size() {
  let cut: DecodedTexture = create_texture(256, 128).without_levels(2);

  assert_eq!((cut.width, cut.height, cut.levels.len()), (64, 32, 4));
  assert_eq!(cut.levels[0].len(), 64 * 32);
}

#[test]
fn texture_lod_keeps_a_block_texture_whole_blocks_and_its_last_level() {
  let small: DecodedTexture = create_texture(8, 8).without_levels(2);

  assert_eq!((small.width, small.height, small.levels.len()), (4, 4, 1));

  let single: DecodedTexture = create_texture(4, 4).without_levels(2);

  assert_eq!((single.width, single.levels.len()), (4, 1));
}
