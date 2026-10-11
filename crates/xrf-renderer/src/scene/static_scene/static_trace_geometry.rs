/// One deferred section of a sector's baked geometry as the level's acceleration structure takes it: an index range of
/// the shared index arena over vertices of the baked layout's positions, and the surface a hit on it is shaded with.
#[derive(Clone, Copy, Debug)]
pub struct StaticTraceGeometry {
  /// Its first index, in the shared index arena.
  pub first_index: u32,
  pub index_count: u32,
  /// Its geometry's first vertex in the baked layout's arena, which its indices count from, and how many it has.
  pub vertex_start: u32,
  pub vertex_count: u32,
  pub surface: u32,
  /// Whether its texels at or below its surface's alpha reference are holes a ray passes through.
  pub is_cut_out: bool,
}
