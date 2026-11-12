use glam::Vec3;
use xrf_math::EPS;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::ray::ray_crossing::RayCrossing;

/// A triangle by its three corners, in the engine's arithmetic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
  pub corners: [Vec3; 3],
}

impl Triangle {
  pub const fn new(corners: [Vec3; 3]) -> Self {
    Self { corners }
  }

  /// Where a ray crosses the triangle within its range, from the sides `facing` names: `CDB::TestRayTri` as the
  /// engine's ray query runs it (`CDB::COLLIDER::_tri`), with that query's range test (`_prim`). A crossing whose
  /// determinant is under `EPS` (`-EPS` to `EPS` from either side) is none: the ray runs along the triangle's plane, or
  /// the triangle is too small to tell.
  pub fn cross(&self, ray: &Ray, facing: Facing) -> Option<RayCrossing> {
    let [first, second, third] = self.corners;
    let edge1: Vec3 = second - first;
    let edge2: Vec3 = third - first;
    let pvec: Vec3 = ray.direction.cross(edge2);
    let det: f32 = edge1.dot(pvec);
    let tvec: Vec3 = ray.origin - first;

    let crossing: RayCrossing = match facing {
      Facing::Front => {
        if det < EPS {
          return None;
        }

        let u: f32 = tvec.dot(pvec);

        if !(0.0..=det).contains(&u) {
          return None;
        }

        let qvec: Vec3 = tvec.cross(edge1);
        let v: f32 = ray.direction.dot(qvec);

        if v < 0.0 || u + v > det {
          return None;
        }

        let inverse: f32 = 1.0 / det;

        RayCrossing {
          distance: edge2.dot(qvec) * inverse,
          u: u * inverse,
          v: v * inverse,
        }
      }
      Facing::Both => {
        if det > -EPS && det < EPS {
          return None;
        }

        let inverse: f32 = 1.0 / det;
        let u: f32 = tvec.dot(pvec) * inverse;

        if !(0.0..=1.0).contains(&u) {
          return None;
        }

        let qvec: Vec3 = tvec.cross(edge1);
        let v: f32 = ray.direction.dot(qvec) * inverse;

        if v < 0.0 || u + v > 1.0 {
          return None;
        }

        RayCrossing {
          distance: edge2.dot(qvec) * inverse,
          u,
          v,
        }
      }
    };

    ray.is_within(crossing.distance).then_some(crossing)
  }

  /// Whether the triangle overlaps the box centred at `center` reaching `half` along each axis, by the separating axis
  /// test (Akenine-Möller), as `CDB::COLLIDER`'s box query runs it under `OPT_FULL_TEST`: the box's three axes, the
  /// nine cross products of an axis with an edge, and the triangle's plane. Touching counts.
  pub fn overlaps_box(&self, center: Vec3, half: Vec3) -> bool {
    let corners: [Vec3; 3] = self.corners.map(|corner| corner - center);

    for axis in 0..3 {
      let (low, high) = Self::get_extent(corners.map(|corner| corner[axis]));

      if low > half[axis] || high < -half[axis] {
        return false;
      }
    }

    let edges: [Vec3; 3] = [
      corners[1] - corners[0],
      corners[2] - corners[1],
      corners[0] - corners[2],
    ];

    for edge in edges {
      for axis in [Vec3::X.cross(edge), Vec3::Y.cross(edge), Vec3::Z.cross(edge)] {
        let (low, high) = Self::get_extent(corners.map(|corner| corner.dot(axis)));
        let radius: f32 = half.dot(axis.abs());

        if low > radius || high < -radius {
          return false;
        }
      }
    }

    let normal: Vec3 = edges[0].cross(edges[1]);

    normal.dot(corners[0]).abs() <= half.dot(normal.abs())
  }

  /// `Fvector::mknormal`: the unit normal of `(p1 - p0) x (p2 - p1)`, which points the way a front face's does, left as
  /// it is when too short to scale (`normalize_safe`).
  pub fn get_normal(&self) -> Vec3 {
    let [first, second, third] = self.corners;
    let normal: Vec3 = (second - first).cross(third - second);
    let length_sqr: f32 = normal.length_squared();

    if length_sqr > f32::MIN_POSITIVE {
      normal * (1.0 / length_sqr).sqrt()
    } else {
      normal
    }
  }

  /// The least and the greatest of three values.
  fn get_extent(values: [f32; 3]) -> (f32, f32) {
    (
      values[0].min(values[1]).min(values[2]),
      values[0].max(values[1]).max(values[2]),
    )
  }
}
