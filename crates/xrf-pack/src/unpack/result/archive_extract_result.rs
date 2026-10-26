use serde::Serialize;

/// What extracting one archived file produced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExtractResult {
  pub name: String,
  pub destination: String,
  pub size: u64,
}
