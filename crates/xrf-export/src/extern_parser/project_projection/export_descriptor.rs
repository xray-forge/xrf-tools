use super::{ExportContractDescriptor, ExportSourceDescriptor};

/// One extern declaration projected for the application-facing exports project.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDescriptor {
  pub name: String,
  pub description: Option<String>,
  pub source: ExportSourceDescriptor,
  #[serde(flatten)]
  pub contract: ExportContractDescriptor,
}
