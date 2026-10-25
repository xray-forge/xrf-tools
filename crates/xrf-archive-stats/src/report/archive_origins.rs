use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;
use crate::report::archive_source_usage::ArchiveSourceUsage;

/// Where a subject's files come from, and what its own search order hides.
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
  /// One entry per source, in search priority order, so the row above is the one that wins.
  pub sources: Vec<ArchiveSourceUsage>,
}
