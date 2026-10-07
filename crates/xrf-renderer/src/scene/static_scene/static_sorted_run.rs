use glam::Vec4;

use crate::scene::static_scene::static_layout::StaticLayout;

/// A run of a level's strictly sorted composited clusters, which a view draws back to front by its sphere after the
/// particles, as the engine draws its `mapSorted` surfaces: a sector section's, standing at the origin, or one place's of
/// an instanced mesh.
#[derive(Clone, Copy, Debug)]
pub struct StaticSortedRun {
  /// Its bounding sphere in the world.
  pub sphere: Vec4,
  /// The place it stands at, counted from its item's own first while it is gathered; none for the origin.
  pub place: Option<u32>,
  pub layout: StaticLayout,
  /// Its first cluster, counted from its item's own first while it is gathered, and how many.
  pub clusters: (u32, u32),
}
