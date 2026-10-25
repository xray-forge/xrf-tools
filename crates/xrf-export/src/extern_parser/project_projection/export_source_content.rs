/// The source text that declares one extern.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSourceContent {
  pub name: String,
  pub path: String,
  pub line: usize,
  pub end_line: usize,
  pub content: String,
}
