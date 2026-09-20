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
