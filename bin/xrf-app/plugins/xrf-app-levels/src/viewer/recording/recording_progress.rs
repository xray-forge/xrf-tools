use serde::{Deserialize, Serialize};

/// Where a recording stands, sent a few times a second while it runs: the seconds of it written since its first frame,
/// its frames, and the frames and sound blocks dropped, the recorder having fallen behind. Its last, where it ended by
/// itself, says why.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct RecordingProgress {
  pub elapsed: f64,
  pub frames: u32,
  pub dropped_frames: u64,
  pub dropped_sound: u64,
  pub ended: Option<String>,
}
