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

impl LtxStructureParseError {
  /// The record for where the parser stopped, as `XrfError::LtxParse` reports it.
  ///
  /// Positions saturate rather than wrap, so an absurd position never names a wrong line.
  pub(crate) fn new(line: usize, column: usize, message: String) -> Self {
    Self {
      column: u32::try_from(column).unwrap_or(u32::MAX),
      line: u32::try_from(line).unwrap_or(u32::MAX),
      message,
    }
  }
}
