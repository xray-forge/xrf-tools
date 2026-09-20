use serde::Serialize;
use xrf_math::Vector3d;

/// Axis aligned box in three.js space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBox {
  pub min: Vector3d,
  pub max: Vector3d,
}
