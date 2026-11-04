use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A burst's video written: its file, the frames in it and the last one's end in seconds, the size it was written at,
/// and the H.264 encoder that wrote it, by its name.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BurstVideoOutcome {
  pub file: PathBuf,
  pub frames: u32,
  pub duration: f64,
  pub width: u32,
  pub height: u32,
  pub encoder: String,
}
