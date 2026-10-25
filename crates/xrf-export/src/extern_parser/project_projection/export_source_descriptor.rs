/// Project-relative source location of an extern declaration.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSourceDescriptor {
  pub path: String,
  pub line: usize,
  pub column: usize,
  /// Last line of the declaration, inclusive, so its body can be fetched without parsing again.
  pub end_line: usize,
}
