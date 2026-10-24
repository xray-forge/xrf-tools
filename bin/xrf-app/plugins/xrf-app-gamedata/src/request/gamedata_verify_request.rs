use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngineChoice;
use xrf_gamedata::GamedataVerificationType;

/// What a verification was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifyRequest {
  /// Gamedata root to verify.
  pub root: PathBuf,
  /// Checks to run, or nothing for every one this build knows.
  pub checks: Option<Vec<GamedataVerificationType>>,
  /// Whether a check that would warn should fail instead.
  pub is_strict: bool,
  /// The engine the tree is meant for, which configs the engines read differently are read as; detected on `auto`.
  pub engine: XrayEngineChoice,
}
