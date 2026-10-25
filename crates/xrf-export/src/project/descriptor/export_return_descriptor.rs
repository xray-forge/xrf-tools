/// The return contract of a callable extern.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReturnDescriptor {
  pub typing: String,
  pub description: Option<String>,
}
