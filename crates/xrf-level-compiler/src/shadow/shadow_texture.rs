use xrf_dds::GrayImage;
use xrf_math::Vector2d;

/// A texture a light ray is dimmed through: its alpha as the SDK's compilers read it, one byte a texel, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShadowTexture {
  /// Its name as the compilers find it, `build.prj`'s cut at the first dot and lowered: `trees\trees_elka`.
  pub name: String,
  pub alpha: GrayImage,
}

impl ShadowTexture {
  /// The alpha at texture coordinates `coordinates`, `alpha / 255` at the nearest texel, the coordinates wrapped
  /// (`getLastRP_Scale`, `xrLC_Light/detail_slot_calculate.cpp`).
  pub(crate) fn get_opacity(&self, coordinates: Vector2d) -> f32 {
    let column: u32 = Self::get_texel(coordinates.x, self.alpha.width());
    let row: u32 = Self::get_texel(coordinates.y, self.alpha.height());

    f32::from(self.alpha.get_pixel(column, row).0[0]) / 255.0
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
