use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A written burst sound: its file, sample rate and channels, and the sound frame each burst frame starts at, in write
/// order.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct BurstSoundOutcome {
  pub file: PathBuf,
  pub rate: u32,
  pub channels: u16,
  pub offsets: Vec<u64>,
}
