/// A rotation, as the format stores one: a quaternion, dequantised.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}
