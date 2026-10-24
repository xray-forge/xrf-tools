use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;
use xrf_vfs::XrayRoots;

/// What opening a configs project for browsing was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsOpenRequest {
  pub session_id: SessionId,
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether to resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}
