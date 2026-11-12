use glam::Vec3;

use crate::ray::ray::Ray;
use crate::tracer::trace_hit::TraceHit;
use crate::triangle::triangle::Triangle;

/// The ray and box queries over a fixed set of triangles that the engine's collision database (`CDB`) answers, each
/// face named by its index in the triangles given. Every ray test is [`Triangle::cross`]; which faces a query visits
/// first is the implementation's, the same on every run.
pub trait Tracer {
  /// How many triangles it holds.
  fn get_face_count(&self) -> usize;

  /// A face's corners.
  ///
  /// # Panics
  ///
  /// Panics when the tracer holds no such face.
  fn get_triangle(&self, face: u32) -> Triangle;

  /// Visits every face whose front a ray meets within its range: the engine's ray query under `OPT_CULL`, all results
  /// kept.
  fn for_each_hit(&self, ray: &Ray, visit: impl FnMut(TraceHit));

  /// [`Self::for_each_hit`] for many rays at once, each hit with the index of its ray in `rays`: faster where the rays
  /// run close together, such as one point set's rays toward one light.
  fn for_each_packet_hit(&self, rays: &[Ray], visit: impl FnMut(usize, TraceHit));

  /// Visits every face overlapping the box centred at `center` reaching `half` along each axis, touching it included:
  /// the engine's box query under `OPT_FULL_TEST`.
  fn for_each_face_in_box(&self, center: Vec3, half: Vec3, visit: impl FnMut(u32));

  /// The nearest face whose front a ray meets within its range: the engine's ray query under
  /// `OPT_ONLYNEAREST | OPT_CULL`.
  fn get_nearest_hit(&self, ray: &Ray) -> Option<TraceHit>;

  /// Whether a ray meets any face within its range, from either side: the engine's ray query under `OPT_ONLYFIRST`.
  fn is_blocked(&self, ray: &Ray) -> bool;

  /// The face the line through `point` along `direction` crosses nearest the point, within `reach` either side and
  /// from either side: what a surface lying on the triangles lies on, whichever way they were wound, before a face
  /// stacked a little above or below it.
  fn get_face_nearest_along(&self, point: Vec3, direction: Vec3, reach: f32) -> Option<u32>;
}
