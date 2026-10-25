use serde::Serialize;
use xrf_archive::ArchiveProject;

use crate::collect::archive_origins_collector::ArchiveOriginsCollector;
use crate::collect::archive_statistics_collector::ArchiveStatisticsCollector;
use crate::collect::archive_world_statistics_entry::ArchiveWorldStatisticsEntry;
use crate::report::archive_compression::ArchiveCompression;
use crate::report::archive_extension_usage::ArchiveExtensionUsage;
use crate::report::archive_folder_usage::ArchiveFolderUsage;
use crate::report::archive_largest_entry::ArchiveLargestEntry;
use crate::report::archive_origins::ArchiveOrigins;
use crate::report::archive_overview::ArchiveOverview;
use crate::report::archive_size_band::ArchiveSizeBand;
use crate::report::archive_volume_summary::ArchiveVolumeSummary;

/// Everything a breakdown of one open subject reports.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStatistics {
  pub overview: ArchiveOverview,
  /// One entry per extension found, heaviest first.
  pub extensions: Vec<ArchiveExtensionUsage>,
  /// One entry per top-level folder, heaviest first.
  pub folders: Vec<ArchiveFolderUsage>,
  /// The size distribution, in ascending band order, with empty bands omitted.
  pub sizes: Vec<ArchiveSizeBand>,
  /// The largest files, largest first.
  pub largest: Vec<ArchiveLargestEntry>,
  /// What the payloads compress to, for a volume set. `None` for a world, which records no stored size.
  pub compression: Option<ArchiveCompression>,
  /// One entry per volume, in merge order. `None` for a world, whose sources are mounts rather than volumes.
  pub volumes: Option<Vec<ArchiveVolumeSummary>>,
  /// Where the files come from and what the search order hides. Answered by both subjects: a volume set is an ordered
  /// stack of volumes exactly as a world is an ordered stack of mounts.
  pub origins: ArchiveOrigins,
}

impl ArchiveStatistics {
  /// Breaks down one volume set.
  pub fn of_volumes(project: &ArchiveProject) -> Self {
    ArchiveStatisticsCollector::collect(project.files.values()).into_report(
      project.archives.len() as u64,
      Some(project.archives.iter().map(ArchiveVolumeSummary::from).collect()),
      ArchiveOriginsCollector::collect_volumes(project),
    )
  }

  /// Breaks down one mounted world.
  ///
  /// `mounts` counts the sources the overview reports; `sources` names them at the grain a container does, which is
  /// finer — one merged mount answers out of each of its volumes.
  pub fn of_world<E: ArchiveWorldStatisticsEntry>(entries: &[E], mounts: &[String], sources: &[String]) -> Self {
    ArchiveStatisticsCollector::collect(entries.iter()).into_report(
      mounts.len() as u64,
      None,
      ArchiveOriginsCollector::collect_world(entries, sources),
    )
  }
}
