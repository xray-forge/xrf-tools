use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::viewer::burst::BurstSoundOutcome;

/// A burst written: its folder, the scene's seconds each frame was drawn at in the order written, the sizes of its
/// frames and of their marks, and its sound where it was asked to record one.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BurstOutcome {
  pub folder: PathBuf,
  pub times: Vec<f32>,
  pub width: u32,
  pub height: u32,
  pub marks_width: u32,
  pub marks_height: u32,
  pub sound: Option<BurstSoundOutcome>,
}
