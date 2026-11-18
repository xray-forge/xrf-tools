use xrf_dds::GrayImage;
use xrf_math::Vector2d;

use crate::shadow::shadow_texture::ShadowTexture;

fn texture(width: u32, height: u32) -> ShadowTexture {
  // Each texel's alpha is its index, so a lookup names the texel it took.
  ShadowTexture {
    name: String::from("test"),
    alpha: GrayImage::from_raw(width, height, (0..width * height).map(|index| index as u8).collect())
      .expect("expect a texel each"),
  }
}

fn texel(texture: &ShadowTexture, x: f32, y: f32) -> u8 {
  (texture.get_opacity(Vector2d { x, y }) * 255.0).round() as u8
}

#[test]
fn takes_the_nearest_texel_rounding_halves_up() {
  let texture: ShadowTexture = texture(4, 4);

  assert_eq!(texel(&texture, 0.0, 0.0), 0);
  assert_eq!(texel(&texture, 0.124, 0.0), 0);
  assert_eq!(texel(&texture, 0.125, 0.0), 1);
  assert_eq!(texel(&texture, 0.5, 0.25), 6);
}

#[test]
fn wraps_coordinates_past_either_edge() {
  let texture: ShadowTexture = texture(4, 4);

  assert_eq!(texel(&texture, 1.0, 0.0), 0);
  assert_eq!(texel(&texture, 1.25, 2.5), 9);
  assert_eq!(texel(&texture, -0.25, -0.25), 15);
}

#[test]
fn wraps_a_negative_texel_as_its_unsigned_remainder_on_other_sides() {
  // -1 is 2^32 - 1 as an unsigned value, which leaves 0 by 3 where a Euclidean remainder leaves 2.
  let texture: ShadowTexture = texture(3, 1);

  assert_eq!(texel(&texture, -1.0 / 3.0, 0.0), 0);
}

#[test]
fn stops_light_by_the_texel_s_alpha() {
  let texture: ShadowTexture = ShadowTexture {
    name: String::from("test"),
    alpha: GrayImage::from_raw(2, 1, vec![0, 255]).expect("expect two texels"),
  };

  assert_eq!(texture.get_opacity(Vector2d { x: 0.0, y: 0.0 }), 0.0);
  assert_eq!(texture.get_opacity(Vector2d { x: 0.5, y: 0.0 }), 1.0);
}
