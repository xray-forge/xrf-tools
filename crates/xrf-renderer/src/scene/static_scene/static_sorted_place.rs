use glam::{Vec3, Vec4};

use crate::scene::static_scene::static_layout::StaticLayout;

/// What a view draws back to front by its distance after the particles, as the engine sorts its blended objects
/// (`mapSorted`), rather than in the order a cull lists them: a spawned object whose model's surfaces are composited
/// over the frame, or a run of a level's strictly sorted composited clusters.
pub struct StaticSortedPlace<'a> {
  /// Its bounding sphere in the world.
  pub sphere: Vec4,
  pub place: u32,
  /// The layout its clusters' vertices are in, whose program draws them.
  pub layout: StaticLayout,
  /// The runs of clusters its composited parts are cut into, as first cluster and count, in their parts' order.
  pub clusters: &'a [(u32, u32)],
}

impl StaticSortedPlace<'_> {
  pub fn get_distance(&self, eye: Vec3) -> f32 {
    self.sphere.truncate().distance(eye)
  }
}
