use serde::Serialize;

use crate::resolved::LtxResolvedDiagnostic;

/// Every section one root resolved to, named and counted but not carried.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedIndex {
  /// Engine identity of the entry point this resolution was produced from.
  pub entry: String,
  /// How the dialect that produced it names itself.
  pub dialect: String,
  /// Sections in the order the dialect answers them, which is authored order under standard LTX and name order under
  /// DLTX. Not re-sorted: that order is the engine's own output, not a presentation choice.
  pub sections: Vec<LtxResolvedIndexEntry>,
  pub diagnostics: Vec<LtxResolvedDiagnostic>,
}

/// One resolved section as the index lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedIndexEntry {
  pub name: String,
  /// Parents the header declared, read back from the declaring config.
  ///
  /// Not from the resolution: flattening inheritance is what resolving does, so a resolved section no longer records
  /// what it inherited from.
  pub parents: Vec<String>,
  pub field_count: usize,
  /// Engine identity of the config whose header declared the section, where the dialect stamped one.
  pub origin: Option<String>,
}
