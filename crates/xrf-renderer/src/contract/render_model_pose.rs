use serde::{Deserialize, Serialize};

/// How a viewport's skinned models stand: a frame of a motion of theirs, or their bind pose, and the bones collapsed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderModelPose {
  /// The motion, by its name, or none for the bind pose.
  pub motion: Option<String>,
  /// Which of its frames; one outside it shows the bind pose.
  pub frame: u32,
  /// Bones collapsed to nothing, by index, each one's descendants among them.
  pub hidden_bones: Vec<u32>,
}
