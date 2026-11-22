use xrf_math::glam::{DVec2, DVec3};

/// One static triangle of the level: its texture in the library's table, corners, and where it is read from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelInstanceTriangle {
  pub(crate) texture: u32,
  pub(crate) points: [DVec3; 3],
  pub(crate) uvs: [DVec2; 3],
  /// Its visual, by its place in the list the index was built from, and its place among the visual's triangles.
  pub(crate) visual: u32,
  pub(crate) index: u32,
  pub(crate) is_claimed: bool,
}
