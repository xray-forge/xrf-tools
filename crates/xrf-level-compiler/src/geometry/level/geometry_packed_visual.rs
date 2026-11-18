use xrf_level::LevelGeomRange;

/// Where one visual's geometry went: its vertices and indices in `level.geom`, its fast path's in `level.geomx`, and a
/// progressive multi-use visual's windows in `level.geom`'s table as `slide_window`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GeometryPackedVisual {
  pub vertices: LevelGeomRange,
  pub indices: LevelGeomRange,
  pub fast_vertices: Option<LevelGeomRange>,
  pub(crate) fast_indices: Option<LevelGeomRange>,
  pub(crate) slide_window: Option<u32>,
}
