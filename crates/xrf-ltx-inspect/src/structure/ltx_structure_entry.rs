use serde::Serialize;

/// One key written before the first section header, which is how a list config declares its members.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureEntry {
  /// One-based line the key was written on.
  pub line: u32,
  /// The key as written, which for a list is the whole line.
  pub name: String,
  /// Whether the line spelled a value at all: a list declares bare names, anything else writes `name = value`.
  pub has_value: bool,
}
