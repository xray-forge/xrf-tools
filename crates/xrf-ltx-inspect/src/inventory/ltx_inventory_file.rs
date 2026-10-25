use serde::Serialize;

use crate::inventory::LtxInventoryRole;

/// One config, and what the project makes of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxInventoryFile {
  /// Lower-case, backslash-separated engine identity.
  pub path: String,
  /// The mount that supplied it, as that mount describes itself - a directory, or an archive volume set.
  pub source: String,
  /// Whether a loose file backs it, which is what decides if an editor could ever write to it.
  pub is_physical: bool,
  /// Configs whose `#include` names it, in project order.
  pub included_by: Vec<String>,
  pub role: LtxInventoryRole,
}
