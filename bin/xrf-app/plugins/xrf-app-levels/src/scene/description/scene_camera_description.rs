use serde::Serialize;
use xrf_editable_level::EditableLevelCamera;
use xrf_math::Vector3d;

/// Where the Level Editor's camera stood when the scene was saved, in the scene's own space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneCameraDescription {
  pub position: Vector3d,
  /// Heading, pitch and bank, in radians.
  pub rotation: Vector3d,
}

impl From<&EditableLevelCamera> for SceneCameraDescription {
  fn from(camera: &EditableLevelCamera) -> Self {
    Self {
      position: camera.position,
      rotation: camera.rotation,
    }
  }
}
