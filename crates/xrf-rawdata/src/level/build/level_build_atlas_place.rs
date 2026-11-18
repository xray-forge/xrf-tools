use xrf_math::Vector2d;

/// Where one texture stands in an atlas: its corner and size as fractions of the atlas, and whether it was turned a
/// quarter to fit, its rows becoming the atlas's columns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelBuildAtlasPlace {
  pub(crate) offset: Vector2d,
  pub(crate) scale: Vector2d,
  pub(crate) is_rotated: bool,
}

impl LevelBuildAtlasPlace {
  /// A texture coordinate moved to the texture's place in the atlas, `MergedTextureRemapUV`.
  pub(crate) fn remap(&self, u: f32, v: f32) -> (f32, f32) {
    let (u, v) = if self.is_rotated { (v, u) } else { (u, v) };

    (self.scale.x * u + self.offset.x, self.scale.y * v + self.offset.y)
  }
}
