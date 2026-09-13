use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// How much of a subject one top-level folder accounts for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFolderUsage {
  /// The first path segment, or `None` for files sitting at the root of the tree.
  pub folder: Option<String>,
  pub measure: ArchiveMeasure,
  /// Stored bytes, for a subject that records them. `None` for a mounted world.
  pub size_compressed: Option<u64>,
}
