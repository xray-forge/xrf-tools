use crate::ray::ray_crossing::RayCrossing;

/// A face a ray meets: its index among the triangles the tracer was built over, how far along the ray, and the
/// barycentrics `u` and `v` there, the weights of its second and third corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceHit {
  pub face: u32,
  pub distance: f32,
  pub u: f32,
  pub v: f32,
}

impl TraceHit {
  pub const fn new(face: u32, crossing: RayCrossing) -> Self {
    Self {
      face,
      distance: crossing.distance,
      u: crossing.u,
      v: crossing.v,
    }
  }
}
