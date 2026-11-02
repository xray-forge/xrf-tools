use ddsfile::{Caps2, D3DFormat, Dds, NewD3dParams};

use crate::scene::texture::decoded_texture::DecodedTexture;

// A header may claim any number of levels; only as many as halve the larger side down to one texel are read.
#[test]
fn counts_the_levels_a_size_can_hold() {
  assert_eq!(DecodedTexture::get_chain_length(1, 1), 1);
  assert_eq!(DecodedTexture::get_chain_length(0, 0), 1);
  assert_eq!(DecodedTexture::get_chain_length(256, 64), 9);
  assert_eq!(DecodedTexture::get_chain_length(5, 3), 3);
  assert_eq!(DecodedTexture::get_chain_length(u32::MAX, 1), 32);
}

// A shader reads a volume's slices as an array, a cube's faces as a cube, and anything else flat.
#[test]
fn reads_a_volume_as_an_array_and_a_cube_as_a_cube() {
  let texture = |layers: u32, is_volume: bool| DecodedTexture {
    format: wgpu::TextureFormat::Rgba8Unorm,
    width: 1,
    height: 1,
    layers,
    is_volume,
    levels: vec![vec![0; 4 * layers as usize]],
    layout: String::from("test"),
    is_expanded: false,
  };

  assert_eq!(texture(1, false).get_view_dimension(), wgpu::TextureViewDimension::D2);
  assert_eq!(
    texture(DecodedTexture::CUBE_FACES, false).get_view_dimension(),
    wgpu::TextureViewDimension::Cube
  );
  assert_eq!(
    texture(6, true).get_view_dimension(),
    wgpu::TextureViewDimension::D2Array
  );
  assert_eq!(
    texture(16, true).get_view_dimension(),
    wgpu::TextureViewDimension::D2Array
  );
}

/// A DXT1 file of a size, its whole chain zeros, with `caps2` saying what it is.
fn create_dxt1(size: (u32, u32), depth: Option<u32>, levels: u32, caps2: Option<Caps2>) -> Dds {
  Dds::new_d3d(NewD3dParams {
    width: size.0,
    height: size.1,
    depth,
    format: D3DFormat::DXT1,
    mipmap_levels: Some(levels),
    caps2,
  })
  .expect("a DXT1 file")
}

fn to_bytes(dds: &Dds) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  dds.write(&mut bytes).expect("a written DDS");

  bytes
}

// A file whose chain stops before the levels its header names is drawn from the levels it holds.
#[test]
fn keeps_the_whole_levels_of_a_chain_cut_short_in_its_file() {
  let mut dds: Dds = create_dxt1((16, 16), None, 5, None);

  // 16x16 is 128 bytes of BC1 and 8x8 is 32: the file stops partway into the third level.
  dds.data.truncate(128 + 32 + 4);

  let decoded: DecodedTexture = DecodedTexture::from_dds(&to_bytes(&dds)).expect("a decoded texture");

  assert_eq!(decoded.levels.len(), 2);
  assert!(!decoded.is_expanded);
}

// A block texture's base must be whole blocks to upload as stored, so a smaller one is expanded to texels.
#[test]
fn expands_a_block_texture_whose_base_is_not_whole_blocks() {
  let decoded: DecodedTexture =
    DecodedTexture::from_dds(&to_bytes(&create_dxt1((6, 6), None, 1, None))).expect("a decoded texture");

  assert!(decoded.is_expanded);
  assert_eq!((decoded.width, decoded.height), (6, 6));
  assert_eq!(decoded.format, wgpu::TextureFormat::Rgba8Unorm);
}

#[test]
fn reads_a_cube_as_six_faces_and_a_volume_as_a_layer_a_slice() {
  let mut cube: Dds = create_dxt1((4, 4), None, 1, Some(Caps2::CUBEMAP | Caps2::CUBEMAP_ALLFACES));
  let mut volume: Dds = create_dxt1((4, 4), Some(3), 1, Some(Caps2::VOLUME));

  // One 4x4 block of BC1 is eight bytes: six of them a cube's faces, three a volume's slices.
  cube.data = vec![0; 6 * 8];
  volume.data = vec![0; 3 * 8];

  let cube: DecodedTexture = DecodedTexture::from_dds(&to_bytes(&cube)).expect("a decoded cube");
  let volume: DecodedTexture = DecodedTexture::from_dds(&to_bytes(&volume)).expect("a decoded volume");

  assert_eq!(cube.layers, DecodedTexture::CUBE_FACES);
  assert!(!cube.is_volume);
  assert_eq!(volume.layers, 3);
  assert!(volume.is_volume);
}

/// A one level texture of a format holding these bytes, as many texels as they make.
fn create_flat(format: wgpu::TextureFormat, bytes: Vec<u8>) -> DecodedTexture {
  DecodedTexture {
    format,
    width: 4,
    height: 4,
    layers: 1,
    is_volume: false,
    levels: vec![bytes],
    layout: String::from("test"),
    is_expanded: false,
  }
}

/// A BC3 block whose colour endpoints are these 5:6:5 colours, its alpha and indices all set.
fn create_bc3_block(first: u16, second: u16) -> Vec<u8> {
  [
    [0xff; 8].as_slice(),
    &first.to_le_bytes(),
    &second.to_le_bytes(),
    &[0xff; 4],
  ]
  .concat()
}

// A block compressed texture holds blue where a block's colour endpoints do, whatever its alpha and indices; an eight bit
// one where a texel does, in its own channel order; any other layout is taken to.
#[test]
fn finds_blue_by_a_blocks_endpoints_or_a_texels_channel() {
  let (red, green, blue): (u16, u16, u16) = (0xf800, 0x07e0, 0x0001);
  let bc3 = |blocks: Vec<Vec<u8>>| create_flat(wgpu::TextureFormat::Bc3RgbaUnorm, blocks.concat());

  assert!(!bc3(vec![create_bc3_block(red, green), create_bc3_block(green | red, 0)]).has_blue());
  assert!(bc3(vec![create_bc3_block(red, green), create_bc3_block(red, blue)]).has_blue());
  assert!(bc3(vec![create_bc3_block(blue | red, green)]).has_blue());

  let bc1 = |first: u16| {
    create_flat(
      wgpu::TextureFormat::Bc1RgbaUnorm,
      [first.to_le_bytes().as_slice(), &red.to_le_bytes(), &[0xff; 4]].concat(),
    )
  };

  assert!(!bc1(green).has_blue());
  assert!(bc1(blue).has_blue());

  let opaque_red: Vec<u8> = [255, 0, 0, 255].repeat(16);

  assert!(!create_flat(wgpu::TextureFormat::Rgba8Unorm, opaque_red.clone()).has_blue());
  assert!(create_flat(wgpu::TextureFormat::Bgra8Unorm, opaque_red).has_blue());
  assert!(create_flat(wgpu::TextureFormat::R8Unorm, vec![0; 16]).has_blue());
}
