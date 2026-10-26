use serde::Serialize;

/// Why one config would not parse.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureParseError {
  /// One-based line the parser stopped on.
  pub line: u32,
  /// One-based column the parser stopped on.
  pub column: u32,
  pub message: String,
}
