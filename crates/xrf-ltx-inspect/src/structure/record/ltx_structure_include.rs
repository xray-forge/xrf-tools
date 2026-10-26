use serde::Serialize;

/// One `#include`, and the configs it actually reached.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureInclude {
  /// One-based line the statement was written on.
  pub line: u32,
  /// The path or wildcard mask as authored.
  pub statement: String,
  /// Engine identities the statement resolved to. Empty means it reached nothing.
  pub resolved: Vec<String>,
}
