use glam::Vec3;

use crate::light::light_point::LightPoint;

/// A point lit: where it lies, its normal, and where its shadow rays start, lifted along the normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSamplePoint {
  pub point: Vec3,
  pub normal: Vec3,
  pub lifted: Vec3,
}

impl LightSamplePoint {
  /// The point at `point` facing `normal`, its rays lifted [`LightPoint::POINT_LIFT`] off it.
  pub fn new(point: Vec3, normal: Vec3) -> Self {
    Self {
      point,
      normal,
      lifted: point + normal * LightPoint::POINT_LIFT,
    }
  }
}
