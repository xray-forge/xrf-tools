use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A written burst video: its file, frame count, duration in seconds, size, and the H.264 encoder's name.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct BurstVideoOutcome {
  pub file: PathBuf,
  pub frames: u32,
  pub duration: f64,
  pub width: u32,
  pub height: u32,
  pub encoder: String,
}
