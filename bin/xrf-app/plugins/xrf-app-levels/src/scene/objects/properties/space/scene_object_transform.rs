use serde::Serialize;
use xrf_editable_level::EditableObjectTransform;
use xrf_math::Vector3d;

/// Where a scene object stands, as the editor keeps it, in the scene's own space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectTransform {
  pub position: Vector3d,
  /// Angles about `x`, `y` and `z`, in radians.
  pub rotation: Vector3d,
  pub scale: Vector3d,
}

impl From<&EditableObjectTransform> for SceneObjectTransform {
  fn from(transform: &EditableObjectTransform) -> Self {
    Self {
      position: transform.position,
      rotation: transform.rotation,
      scale: transform.scale,
    }
  }
}
