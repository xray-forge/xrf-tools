use xrf_dds::GrayImage;
use xrf_math::Vector2d;

/// A texture a light ray is dimmed through: its alpha as the SDK's compilers read it, one byte a texel, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowTexture {
  /// Its name as the compilers find it, `build.prj`'s cut at the first dot and lowered: `trees\trees_elka`.
  pub name: String,
  pub alpha: GrayImage,
}

impl ShadowTexture {
  /// How much light passes a hit at texture coordinates `coordinates`: `1 - alpha / 255` at the nearest texel, the
  /// coordinates wrapped (`getLastRP_Scale`, `xrLC_Light/detail_slot_calculate.cpp:146`).
  pub fn get_transmittance(&self, coordinates: Vector2d) -> f32 {
    let column: u32 = Self::get_texel(coordinates.x, self.alpha.width());
    let row: u32 = Self::get_texel(coordinates.y, self.alpha.height());

    1.0 - f32::from(self.alpha.get_pixel(column, row).0[0]) / 255.0
  }

  /// The texel `coordinate` rounds to along a side of `size` texels, wrapped.
  ///
  /// The SDK takes the remainder of an `int` by the `u32` size, so a negative texel wraps as its two's complement
  /// does: the same texel as a Euclidean remainder for a power-of-two side, another for any other side.
  fn get_texel(coordinate: f32, size: u32) -> u32 {
    let texel: i32 = (coordinate * size as f32 + 0.5).floor() as i32;

    texel.cast_unsigned() % size
  }
}

#[cfg(test)]
mod tests {
  use xrf_dds::GrayImage;
  use xrf_math::Vector2d;

  use super::ShadowTexture;

  fn texture(width: u32, height: u32) -> ShadowTexture {
    // Each texel's alpha is its index, so a lookup names the texel it took.
    ShadowTexture {
      name: String::from("test"),
      alpha: GrayImage::from_raw(width, height, (0..width * height).map(|index| index as u8).collect())
        .expect("expect a texel each"),
    }
  }

  fn texel(texture: &ShadowTexture, x: f32, y: f32) -> u8 {
    (((1.0 - texture.get_transmittance(Vector2d { x, y })) * 255.0).round()) as u8
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
  fn passes_light_by_the_texel_s_transparency() {
    let texture: ShadowTexture = ShadowTexture {
      name: String::from("test"),
      alpha: GrayImage::from_raw(2, 1, vec![0, 255]).expect("expect two texels"),
    };

    assert_eq!(texture.get_transmittance(Vector2d { x: 0.0, y: 0.0 }), 1.0);
    assert_eq!(texture.get_transmittance(Vector2d { x: 0.5, y: 0.0 }), 0.0);
  }
}
