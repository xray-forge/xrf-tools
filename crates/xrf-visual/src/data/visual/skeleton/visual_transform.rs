use serde::Serialize;
use xrf_math::Vector3d;

/// One transform in renderer space: three basis vectors and a translation.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualTransform {
  pub i: Vector3d,
  pub j: Vector3d,
  pub k: Vector3d,
  pub c: Vector3d,
}

impl VisualTransform {
  /// Floats one transform takes: its basis, then its translation.
  pub const FLOATS: usize = 12;

  /// A transform out of its twelve floats, basis then translation, as a baked motion's frame holds one.
  pub fn from_floats(floats: &[f32; Self::FLOATS]) -> Self {
    Self {
      i: Vector3d::new(floats[0], floats[1], floats[2]),
      j: Vector3d::new(floats[3], floats[4], floats[5]),
      k: Vector3d::new(floats[6], floats[7], floats[8]),
      c: Vector3d::new(floats[9], floats[10], floats[11]),
    }
  }

  /// Its twelve floats, basis then translation.
  pub fn to_floats(&self) -> [f32; Self::FLOATS] {
    let Self { i, j, k, c } = self;

    [i.x, i.y, i.z, j.x, j.y, j.z, k.x, k.y, k.z, c.x, c.y, c.z]
  }
}
