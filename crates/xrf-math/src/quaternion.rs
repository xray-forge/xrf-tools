use serde::{Deserialize, Serialize};

/// A rotation, the engine's `Fquaternion`: the vector part `x`, `y`, `z` and the scalar `w`, in the engine's order.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quaternion {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

impl Quaternion {
  /// No rotation.
  pub const IDENTITY: Self = Self {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
  };
}
