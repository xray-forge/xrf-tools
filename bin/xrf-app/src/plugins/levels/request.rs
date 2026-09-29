use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngine;
use xrf_vfs::XrayRoots;

use crate::plugins::levels::state::LevelSource;

/// What opening a level was asked to read, and how the game's configs beside it are read.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct LevelOpenRequest {
  pub source: LevelSource,
  /// Trees the level and its textures are searched in.
  pub roots: XrayRoots,
  /// Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
  /// Which engine the game's configs, its weather among them, are read as.
  pub engine: XrayEngine,
}
