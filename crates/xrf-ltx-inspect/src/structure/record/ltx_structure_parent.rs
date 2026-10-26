use serde::Serialize;

/// One name after the `:` of a header, and whether the resolution holds it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureParent {
  pub name: String,
  /// Whether a section by this name exists in the resolution this file was read against.
  pub resolves: bool,
}
