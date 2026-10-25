use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// How much of a subject one source accounts for, and how much of itself it loses to the sources above it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSourceUsage {
  /// The volume file, or the loose root, a copy sits in — the grain a container names, not the mount's.
  pub source: String,
  /// Whether this source is a loose tree rather than a volume set.
  pub is_loose: bool,
  /// Entries of this source a lookup reaches.
  pub wins: ArchiveMeasure,
  /// Entries of this source no lookup reaches, because a higher-priority source claims their engine path.
  pub hides: ArchiveMeasure,
}
