use xrf_level::LevelGeomRange;

/// Where one visual's geometry went in the level's buffers: its vertices and indices in `level.geom`, and its fast
/// path's in `level.geomx` where it has one. A progressive multi-use visual's windows go to `level.geom`'s table too,
/// as its `slide_window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryPackedVisual {
  pub vertices: LevelGeomRange,
  pub indices: LevelGeomRange,
  pub fast_vertices: Option<LevelGeomRange>,
  pub(crate) fast_indices: Option<LevelGeomRange>,
  pub(crate) slide_window: Option<u32>,
}
