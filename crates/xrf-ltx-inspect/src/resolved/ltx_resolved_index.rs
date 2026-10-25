use serde::Serialize;

use crate::resolved::{LtxResolvedDiagnostic, LtxResolvedIndexEntry};

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
