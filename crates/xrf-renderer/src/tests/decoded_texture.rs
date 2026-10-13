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
