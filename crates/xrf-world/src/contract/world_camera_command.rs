use serde::{Deserialize, Serialize};

/// What a consumer can ask of the camera it described.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorldCameraCommand {
  /// Back to where the camera started.
  Reset,
  /// Towards the target or away from it, by a multiplier on the distance: above one moves away.
  Dolly { step: f32 },
}
