use crate::scene::static_scene::static_layout::StaticLayout;

/// One deferred geometry as the level's acceleration structures take it: an index range of the shared index arena over
/// vertices of its layout's arena, and the surface a hit on it is shaded with. A sector's baked section, or an
/// instanced mesh's whole detail, in its own space.
#[derive(Clone, Copy, Debug)]
pub struct StaticTraceGeometry {
  pub layout: StaticLayout,
  /// Its first index, in the shared index arena.
  pub first_index: u32,
  pub index_count: u32,
  /// Its geometry's first vertex in its layout's arena, which its indices count from, and how many it has.
  pub vertex_start: u32,
  pub vertex_count: u32,
  pub surface: u32,
  /// Whether its texels at or below its surface's alpha reference are holes a ray passes through.
  pub is_cut_out: bool,
}
