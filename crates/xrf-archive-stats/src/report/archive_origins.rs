use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// How much of a mounted world one source accounts for, and how much of itself it loses to the sources above it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSourceUsage {
  /// The source as it names itself, matching an entry of the world's own mount list.
  pub source: String,
  /// Whether this source is a loose tree rather than a volume set.
  pub is_loose: bool,
  /// Entries of this source a lookup reaches.
  pub wins: ArchiveMeasure,
  /// Entries of this source no lookup reaches, because a higher-priority source claims their engine path.
  pub hides: ArchiveMeasure,
}

/// Where a mounted world's files come from, and what its mount order hides.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOrigins {
  /// Winning entries served from a loose file on disk.
  pub loose: ArchiveMeasure,
  /// Winning entries served from inside a volume.
  pub archived: ArchiveMeasure,
  /// Entries no lookup reaches, across every source.
  pub hidden: ArchiveMeasure,
  /// One entry per source, in mount priority order, so the row above is the one that wins.
  pub sources: Vec<ArchiveSourceUsage>,
}
