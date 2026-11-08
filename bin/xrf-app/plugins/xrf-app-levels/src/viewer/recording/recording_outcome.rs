use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A recording written: its file, its length in seconds, its frames, the frames and sound blocks dropped, its size, and
/// the H.264 encoder that wrote it, by its name.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordingOutcome {
  pub file: PathBuf,
  pub duration: f64,
  pub frames: u32,
  pub dropped_frames: u64,
  pub dropped_sound: u64,
  pub width: u32,
  pub height: u32,
  pub encoder: String,
}
