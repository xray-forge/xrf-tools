use xrf_math::{Vector2d, Vector3d};

/// One face gathered: its corners placed, their welded points, their normals turned into the level, their texture
/// coordinates, and its surface by index.
pub(crate) struct LevelBuildPartFace {
  pub(crate) corners: [Vector3d; 3],
  pub(crate) points: [u32; 3],
  pub(crate) normals: [Vector3d; 3],
  pub(crate) texture_coordinates: [Vector2d; 3],
  pub(crate) surface: usize,
}
