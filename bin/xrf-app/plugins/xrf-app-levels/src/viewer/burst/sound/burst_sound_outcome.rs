use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A burst's sound written: its file, its frames a second and channels, and the sound's frame each frame read back
/// starts at, in the order written.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct BurstSoundOutcome {
  pub file: PathBuf,
  pub rate: u32,
  pub channels: u16,
  pub offsets: Vec<u64>,
}
