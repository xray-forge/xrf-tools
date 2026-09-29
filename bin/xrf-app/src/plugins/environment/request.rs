use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngine;
use xrf_vfs::XrayRoots;

/// Which game's environment configs to read, and as which engine.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Whether to resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
  pub engine: XrayEngine,
}
