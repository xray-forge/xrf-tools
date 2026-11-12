/// Where a ray crosses a triangle: how far along the ray, and the barycentrics `u` and `v` of the crossing, the weights
/// of the triangle's second and third corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayCrossing {
  pub distance: f32,
  pub u: f32,
  pub v: f32,
}
