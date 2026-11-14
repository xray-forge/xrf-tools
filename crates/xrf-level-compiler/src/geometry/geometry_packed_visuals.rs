use xrf_level::LevelGeomBuffers;

use crate::geometry::geometry_packed_visual::GeometryPackedVisual;

/// A level's static visuals packed: where each one's geometry went, its shader table entry, the table, and the two
/// geometry files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryPackedVisuals {
  /// Each visual, in visual order.
  pub visuals: Vec<GeometryPackedVisual>,
  /// Each visual's entry in [`Self::shaders`].
  pub shader_ids: Vec<u16>,
  /// The shader table, `fsL_SHADERS`.
  pub shaders: Vec<String>,
  /// `level.geom`.
  pub geometry: LevelGeomBuffers,
  /// `level.geomx`, the fast path's.
  pub fast: LevelGeomBuffers,
}
