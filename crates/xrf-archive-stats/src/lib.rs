#![doc = include_str!("../README.md")]

pub(crate) mod collect;
pub(crate) mod report;

#[cfg(test)]
mod tests;

pub use crate::collect::archive_statistics_entry::{ArchiveStatisticsEntry, ArchiveWorldStatisticsEntry};
pub use crate::report::archive_compression::ArchiveCompression;
pub use crate::report::archive_extension_usage::ArchiveExtensionUsage;
pub use crate::report::archive_folder_usage::ArchiveFolderUsage;
pub use crate::report::archive_largest_entry::ArchiveLargestEntry;
pub use crate::report::archive_measure::ArchiveMeasure;
pub use crate::report::archive_origins::{ArchiveOrigins, ArchiveSourceUsage};
pub use crate::report::archive_overview::ArchiveOverview;
pub use crate::report::archive_size_band::ArchiveSizeBand;
pub use crate::report::archive_statistics::ArchiveStatistics;
pub use crate::report::archive_volume_summary::ArchiveVolumeSummary;
