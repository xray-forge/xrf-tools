use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::viewer::burst::{BurstSoundOutcome, BurstVideoOutcome};

/// A written burst: its folder, each frame's scene time in seconds in write order, the frame and marks sizes, and its
/// sound and video when asked for.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct BurstOutcome {
  pub folder: PathBuf,
  pub times: Vec<f64>,
  pub width: u32,
  pub height: u32,
  pub marks_width: u32,
  pub marks_height: u32,
  pub sound: Option<BurstSoundOutcome>,
  pub video: Option<BurstVideoOutcome>,
}
