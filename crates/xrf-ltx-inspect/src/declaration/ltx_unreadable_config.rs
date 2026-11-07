use serde::Serialize;

use crate::structure::LtxStructureParseError;

/// A config a declaration search could not read, and why.
///
/// Recorded rather than failing the search: one broken config says nothing about what the others declare, and a search
/// that stopped at it would hide every answer behind the one file that cannot give any.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxUnreadableConfig {
  /// Engine identity of the config.
  pub path: String,
  pub error: LtxStructureParseError,
}
