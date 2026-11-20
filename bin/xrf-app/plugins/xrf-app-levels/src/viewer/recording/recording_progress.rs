use serde::{Deserialize, Serialize};

/// A running recording's progress, sent a few times a second: seconds written since its first frame, frames, and the
/// frames and sound blocks dropped when the recorder fell behind. `ended` gives the reason when it ends by itself.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordingProgress {
  pub elapsed: f64,
  pub frames: u32,
  pub dropped_frames: u64,
  pub dropped_sound: u64,
  pub ended: Option<String>,
}
