use xrf_math::{Aabb, EPS_L, Vector3d};

/// A visual's or node's bounds as its header carries them (`OGF_Base::CalcBounds`, `xrLC/OGF_Face_Sphere.cpp:27`):
/// the box of its geometry grown by `EPS_L`, and a sphere around it.
///
/// The box is xrLC's exactly. The sphere is the smaller of two that hold every point within `EPS_L`: the box's centre
/// reaching its farthest point, as xrLC's second candidate is, and Ritter's grown sphere. xrLC also tries two minimal
/// spheres (a miniball and Magic's `MinSphere`) and takes the smallest valid one, so its radius may be smaller; a
/// larger sphere only culls later.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometryBounds {
  pub bounds: Aabb,
  pub center: Vector3d,
  pub radius: f32,
}

impl GeometryBounds {
  /// The bounds of `points`, of which there is at least one.
  #[expect(
    clippy::manual_midpoint,
    reason = "`Fbox::getcenter` sums and halves in single precision, which `f32::midpoint` does not"
  )]
  pub fn of(points: &[Vector3d]) -> Self {
    let mut min: Vector3d = points.first().copied().unwrap_or(Vector3d::ZERO);
    let mut max: Vector3d = min;

    for point in points {
      min = Vector3d::new(min.x.min(point.x), min.y.min(point.y), min.z.min(point.z));
      max = Vector3d::new(max.x.max(point.x), max.y.max(point.y), max.z.max(point.z));
    }

    let bounds: Aabb = Aabb {
      min: Vector3d::new(min.x - EPS_L, min.y - EPS_L, min.z - EPS_L),
      max: Vector3d::new(max.x + EPS_L, max.y + EPS_L, max.z + EPS_L),
    };
    let box_center: Vector3d = Vector3d::new(
      (bounds.min.x + bounds.max.x) * 0.5,
      (bounds.min.y + bounds.max.y) * 0.5,
      (bounds.min.z + bounds.max.z) * 0.5,
    );
    let box_radius: f32 = points
      .iter()
      .map(|point| box_center.distance_squared(point))
      .fold(-1.0, f32::max)
      .abs()
      .sqrt();
    let ritter: Self = Self::get_ritter(points, bounds);

    if ritter.radius < box_radius
      && points
        .iter()
        .all(|point| ritter.center.distance(point) <= ritter.radius + EPS_L)
    {
      ritter
    } else {
      Self {
        bounds,
        center: box_center,
        radius: box_radius,
      }
    }
  }

  /// `bounds` with Ritter's sphere: the farthest pair from the first point's farthest, grown to take in each point
  /// outside it.
  #[expect(
    clippy::manual_midpoint,
    reason = "kept in the same single-precision form as the box's centre beside it"
  )]
  fn get_ritter(points: &[Vector3d], bounds: Aabb) -> Self {
    let farthest_from = |from: &Vector3d| {
      points
        .iter()
        .copied()
        .max_by(|first, second| from.distance_squared(first).total_cmp(&from.distance_squared(second)))
        .unwrap_or(*from)
    };
    let start: Vector3d = points.first().copied().unwrap_or(Vector3d::ZERO);
    let first: Vector3d = farthest_from(&start);
    let second: Vector3d = farthest_from(&first);
    let mut center: Vector3d = Vector3d::new(
      (first.x + second.x) * 0.5,
      (first.y + second.y) * 0.5,
      (first.z + second.z) * 0.5,
    );
    let mut radius: f32 = first.distance(&second) * 0.5;

    for point in points {
      let distance: f32 = center.distance(point);

      if distance > radius {
        let grown: f32 = (radius + distance) * 0.5;
        let step: f32 = (distance - grown) / distance;

        center = Vector3d::new(
          center.x + (point.x - center.x) * step,
          center.y + (point.y - center.y) * step,
          center.z + (point.z - center.z) * step,
        );
        radius = grown;
      }
    }

    Self { bounds, center, radius }
  }
}
