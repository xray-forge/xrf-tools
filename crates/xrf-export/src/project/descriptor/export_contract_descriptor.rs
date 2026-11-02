use super::{ExportParameterDescriptor, ExportReturnDescriptor};

/// The mutually exclusive contracts an extern can expose.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportContractDescriptor {
  Callable {
    parameters: Vec<ExportParameterDescriptor>,
    returns: ExportReturnDescriptor,
  },
  Value {
    typing: String,
  },
}
