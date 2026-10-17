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
