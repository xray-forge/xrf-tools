use xrf_level::LevelGeomRange;

/// Where one visual's geometry went in the level's buffers: its vertices and indices in `level.geom`, and its fast
/// path's in `level.geomx` where it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryPackedVisual {
  pub vertices: LevelGeomRange,
  pub indices: LevelGeomRange,
  pub fast_vertices: Option<LevelGeomRange>,
  pub fast_indices: Option<LevelGeomRange>,
}
