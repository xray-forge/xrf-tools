use xrf_math::Matrix4x4;

use crate::level::geometry::level_visual_geometry::LevelVisualGeometry;

/// One tree visual of a sector, its geometry in its model's own space.
pub(crate) struct LevelTreeVisual {
  /// The visual's `MT_LOD` parent, which names its placement, where it has one.
  pub(crate) parent: Option<u32>,
  pub(crate) sector: u16,
  /// Its vertex buffer, first vertex, index buffer and first index: which subdivision of which model it draws.
  pub(crate) subdivision: [u32; 4],
  pub(crate) placement: Matrix4x4,
  pub(crate) geometry: LevelVisualGeometry,
}
