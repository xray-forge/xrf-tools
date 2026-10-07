use glam::{Vec3, Vec4};

/// A spawned object whose model's surfaces are composited over the frame: drawn back to front by its distance, as the
/// engine sorts its blended objects (`mapSorted`), rather than in the order a cull lists them.
pub struct StaticSortedPlace<'a> {
  /// Its bounding sphere in the world.
  pub sphere: Vec4,
  pub place: u32,
  /// The runs of clusters its composited parts are cut into, as first cluster and count, in their parts' order.
  pub clusters: &'a [(u32, u32)],
}

impl StaticSortedPlace<'_> {
  /// Whether its sphere stands inside every plane of a view, each pointing in.
  pub fn is_in_view(&self, planes: &[Vec4; 6]) -> bool {
    planes
      .iter()
      .all(|plane| plane.truncate().dot(self.sphere.truncate()) + plane.w >= -self.sphere.w)
  }

  pub fn get_distance(&self, eye: Vec3) -> f32 {
    self.sphere.truncate().distance(eye)
  }
}
