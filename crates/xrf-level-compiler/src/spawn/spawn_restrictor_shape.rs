use glam::{Vec3, Vec4};
use xrf_math::{Aabb, Affine3d, Vector3d};

/// One shape of a space restrictor placed in the level, as `CSpaceRestrictorWrapper` tests nodes against it
/// (`space_restrictor_wrapper.cpp`).
pub(crate) enum SpawnRestrictorShape {
  Sphere {
    centre: Vec3,
    radius: f32,
  },
  /// A box as its six planes, each a normal and an offset (`Fplane`), with the corners that bound it.
  Box {
    planes: [Vec4; 6],
    corners: [Vec3; 8],
  },
}

impl SpawnRestrictorShape {
  /// `EPS_L`, the radius every point test passes.
  pub(crate) const EPS_L: f32 = 0.001;

  /// A sphere of the object's shape, `center` and `radius` in its frame, placed by `transform`.
  pub(crate) fn new_sphere(transform: &Affine3d, center: &Vector3d, radius: f32) -> Self {
    Self::Sphere {
      centre: Vec3::from(transform.transform_point(center)),
      radius,
    }
  }

  /// A box of the object's shape, the unit cube taken by `matrix` in its frame, placed by `transform`.
  pub(crate) fn new_box(transform: &Affine3d, matrix: &Affine3d) -> Self {
    let placed: Affine3d = transform.compose(matrix);
    let point = |x: f32, y: f32, z: f32| Vec3::from(placed.transform_point(&Vector3d::new(x, y, z)));
    let corners: [Vec3; 8] = [
      point(-0.5, -0.5, -0.5),
      point(-0.5, -0.5, 0.5),
      point(-0.5, 0.5, 0.5),
      point(-0.5, 0.5, -0.5),
      point(0.5, 0.5, 0.5),
      point(0.5, 0.5, -0.5),
      point(0.5, -0.5, 0.5),
      point(0.5, -0.5, -0.5),
    ];

    Self::Box {
      planes: [
        Self::plane(corners[0], corners[3], corners[5]),
        Self::plane(corners[1], corners[2], corners[3]),
        Self::plane(corners[6], corners[5], corners[4]),
        Self::plane(corners[4], corners[2], corners[1]),
        Self::plane(corners[3], corners[2], corners[4]),
        Self::plane(corners[1], corners[0], corners[6]),
      ],
      corners,
    }
  }

  /// `inside(position, EPS_L)`: a sphere of `EPS_L` at the position meets the shape.
  pub(crate) fn contains(&self, position: Vec3) -> bool {
    match self {
      Self::Sphere { centre, radius } => {
        position.distance_squared(*centre) <= (radius + Self::EPS_L) * (radius + Self::EPS_L)
      }
      Self::Box { planes, .. } => planes
        .iter()
        .all(|plane| plane.truncate().dot(position) + plane.w <= Self::EPS_L),
    }
  }

  /// The box the shape lies within.
  pub(crate) fn get_bounds(&self) -> Aabb {
    match self {
      Self::Sphere { centre, radius } => Aabb {
        min: Vector3d::from(*centre - Vec3::splat(*radius)),
        max: Vector3d::from(*centre + Vec3::splat(*radius)),
      },
      Self::Box { corners, .. } => {
        Aabb::from_points(corners.iter().map(|corner| Vector3d::from(*corner))).expect("a box has corners")
      }
    }
  }

  /// `Fplane::build(v1, v2, v3)`: the normal of the turn from the first edge to the second, and the offset putting the
  /// first point on the plane.
  fn plane(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec4 {
    let normal: Vec3 = (v1 - v2).cross(v1 - v3).normalize();

    normal.extend(-normal.dot(v1))
  }
}
