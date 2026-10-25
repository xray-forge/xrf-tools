use serde::Serialize;

/// One resolved section as the index lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedIndexEntry {
  pub name: String,
  /// Parents the header declared, read back from the declaring config.
  ///
  /// Not from the resolution: flattening inheritance is what resolving does, so a resolved section no longer records
  /// what it inherited from.
  pub parents: Vec<String>,
  pub field_count: usize,
  /// Engine identity of the config whose header declared the section, where the dialect stamped one.
  pub origin: Option<String>,
}
