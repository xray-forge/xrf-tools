use serde::Serialize;
use xrf_vfs::XrayRevision;

use crate::session::SessionChange;

/// What one commit changed, or a derivation landing after it, as every page subscribed to changes is told: the project
/// revision it left, and what it did to each open session that moved or could not.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetChangeEvent {
  pub revision: XrayRevision,
  pub sessions: Vec<SessionChange>,
}
