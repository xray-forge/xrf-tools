use serde::Serialize;
use xrf_math::Vector3d;

/// Enclosing sphere in three.js space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSphere {
  pub center: Vector3d,
  pub radius: f32,
}
