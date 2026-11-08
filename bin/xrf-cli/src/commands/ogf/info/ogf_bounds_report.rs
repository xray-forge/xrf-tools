use serde::Serialize;
use xrf_math::{Aabb, Sphere};

/// A bounding volume, reported as the numbers the file carries rather than as debug text.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OgfBoundsReport {
  maximum: [f32; 3],
  minimum: [f32; 3],
  radius: f32,
  sphere: [f32; 3],
}

impl OgfBoundsReport {
  pub(crate) fn new(bounding_box: &Aabb, bounding_sphere: &Sphere) -> Self {
    Self {
      maximum: [bounding_box.max.x, bounding_box.max.y, bounding_box.max.z],
      minimum: [bounding_box.min.x, bounding_box.min.y, bounding_box.min.z],
      radius: bounding_sphere.radius,
      sphere: [
        bounding_sphere.center.x,
        bounding_sphere.center.y,
        bounding_sphere.center.z,
      ],
    }
  }
}
