use serde::Serialize;

/// One tool of the scene a decompile wrote, and what its part holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileReportTool {
  pub name: String,
  /// The objects its text part holds; none for a binary part.
  pub objects: Option<usize>,
}
