use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// How much of a subject one file extension accounts for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExtensionUsage {
  /// The spelling as found, lower-cased, or `None` for a name carrying no extension at all.
  pub extension: Option<String>,
  /// Whether [`xrf_extension::XrayExtension`] declares this spelling.
  pub is_declared: bool,
  pub measure: ArchiveMeasure,
  /// Stored bytes these entries occupy, for a subject that records them.
  pub size_compressed: Option<u64>,
}
