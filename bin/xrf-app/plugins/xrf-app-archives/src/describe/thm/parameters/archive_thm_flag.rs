use serde::Serialize;

/// One bit of the texture param flag word.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFlag {
  /// SDK identifier, which is the name an author of a `.thm` would recognise.
  pub label: String,
  pub is_set: bool,
}
