use serde::Serialize;

use crate::structure::LtxStructureParseError;

/// A config a declaration search could not read, and why.
///
/// Recorded rather than failing the search, so one broken config does not hide what the others declare.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxUnreadableConfig {
  /// Engine identity of the config.
  pub path: String,
  pub error: LtxStructureParseError,
}
