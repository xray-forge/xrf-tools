use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

/// What a config verification was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsVerifyRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether to resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
  /// Whether to check what only a case-sensitive file system breaks on too, such as an include naming a file in
  /// another case, which the vanilla game data does.
  pub is_strict: bool,
}
