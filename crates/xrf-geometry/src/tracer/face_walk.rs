use glam::Vec3;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::tracer::trace_hit::TraceHit;
use crate::triangle::triangle::Triangle;

/// A tracer's walk over the faces under the boxes a ray reaches, and the ray queries of [`crate::Tracer`] answered
/// from it alike for every tree: each face tested by [`Triangle::cross`], the nearest hit narrowing the reach the walk
/// tests boxes within.
pub(crate) trait FaceWalk {
  /// Each face, by index.
  fn get_triangles(&self) -> &[Triangle];

  /// Visits each face under a box the ray reaches within a reach the visitor may shorten, in the tree's order, until
  /// the visitor returns true.
  fn walk(&self, ray: &Ray, visit: impl FnMut(u32, &mut f32) -> bool);

  /// [`crate::Tracer::for_each_hit_facing`].
  fn walk_hits(&self, ray: &Ray, facing: Facing, mut visit: impl FnMut(TraceHit)) {
    let triangles: &[Triangle] = self.get_triangles();

    self.walk(ray, |face, _| {
      if let Some(crossing) = triangles[face as usize].cross(ray, facing) {
        visit(TraceHit::new(face, crossing));
      }

      false
    });
  }

  /// [`crate::Tracer::get_nearest_hit`]: a nearer hit replaces the kept one and shortens the reach.
  fn walk_nearest_hit(&self, ray: &Ray) -> Option<TraceHit> {
    let triangles: &[Triangle] = self.get_triangles();
    let mut nearest: Option<TraceHit> = None;

    self.walk(ray, |face, reach| {
      let narrowed: Ray = Ray { range: *reach, ..*ray };

      if let Some(crossing) = triangles[face as usize].cross(&narrowed, Facing::Front)
        && nearest.is_none_or(|best| crossing.distance < best.distance)
      {
        nearest = Some(TraceHit::new(face, crossing));
        *reach = crossing.distance;
      }

      false
    });

    nearest
  }

  /// [`crate::Tracer::is_blocked`]: the walk ends at the first face met.
  fn walk_is_blocked(&self, ray: &Ray) -> bool {
    let triangles: &[Triangle] = self.get_triangles();
    let mut is_blocked: bool = false;

    self.walk(ray, |face, _| {
      is_blocked = triangles[face as usize].cross(ray, Facing::Both).is_some();
      is_blocked
    });

    is_blocked
  }

  /// [`crate::Tracer::get_face_nearest_along`], walked from both ends: a line grazing a face's edge can miss it from
  /// one end and cross it from the other.
  fn walk_face_nearest_along(&self, point: Vec3, direction: Vec3, reach: f32) -> Option<u32> {
    let triangles: &[Triangle] = self.get_triangles();
    // The face the line from the far end crosses nearest the point, by how far off it.
    let nearest_to_middle = |along: Vec3| {
      let ray: Ray = Ray::new(point - along * reach, along, reach * 2.0);
      let mut nearest: Option<MiddleCrossing> = None;

      self.walk(&ray, |face, _| {
        if let Some(crossing) = triangles[face as usize].cross(&ray, Facing::Both) {
          let off: f32 = (crossing.distance - reach).abs();

          if nearest.is_none_or(|best| off < best.off) {
            nearest = Some(MiddleCrossing { off, face });
          }
        }

        false
      });

      nearest
    };

    [direction, -direction]
      .into_iter()
      .filter_map(nearest_to_middle)
      .min_by(|first, second| first.off.total_cmp(&second.off))
      .map(|middle| middle.face)
  }
}

/// A face a line crosses, by how far its crossing lies off the line's middle.
#[derive(Clone, Copy, Debug)]
struct MiddleCrossing {
  off: f32,
  face: u32,
}
