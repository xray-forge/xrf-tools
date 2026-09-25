use serde::Serialize;
use xrf_visual::VisualTransform;

/// Where one spawned object stands, and which model it is drawn as.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnPlacement {
  pub name: String,
  pub section: String,
  /// The model, by its index among the description's models.
  pub model: u32,
  /// The object's `XFORM`, in renderer space.
  pub transform: VisualTransform,
}
