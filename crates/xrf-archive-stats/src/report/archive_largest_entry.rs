use serde::Serialize;

/// One of the largest files a subject holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLargestEntry {
  /// Engine identity, as every other surface of the explorer addresses this file by.
  pub name: String,
  /// Unpacked bytes.
  pub size_real: u64,
  /// Stored bytes, for a subject that records them. `None` for a mounted world.
  pub size_compressed: Option<u64>,
}
