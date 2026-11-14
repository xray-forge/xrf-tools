use glam::Vec3;

use crate::ray::ray_group::RayGroup;

/// What a GPU batch reads of a plan of rays: its points, each passing its own face, and its rays' groups.
pub(crate) trait RayGpuPlan {
  /// How many points the plan's rays leave from.
  fn get_point_count(&self) -> usize;

  /// Where point `point`'s rays start, lifted off its face.
  fn get_origin(&self, point: usize) -> Vec3;

  /// Point `point` itself, which a ray toward a light at a position is aimed from.
  fn get_point(&self, point: usize) -> Vec3;

  /// Point `point`'s own face in the ray model, which its rays pass through; none for none.
  fn get_skip(&self, point: usize) -> Option<u32>;

  /// The plan's ray groups, its rays numbered in their order.
  fn get_groups(&self) -> &[RayGroup];

  /// How many rays the groups make.
  fn get_ray_count(&self) -> usize;
}
