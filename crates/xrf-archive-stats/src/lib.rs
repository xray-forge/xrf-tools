//! What an open archive subject holds, broken down the ways a person asks about it.
//!
//! Above both format crates rather than inside either, the way `xrf-pack` sits: the same question is asked of a volume
//! set, whose name table records a stored size per entry, and of a mounted world, which answers loose files and
//! archived entries through one lookup and records no stored size at all. Putting the shared arithmetic in either crate
//! would give one of them knowledge of the other's subject.
//!
//! [`ArchiveStatistics::of_volumes`] and [`ArchiveStatistics::of_world`] are the whole interface. Every part of the
//! report falls out of a single pass over the entries, so there is no per-question call to make and no way to ask for
//! half a report over an installation-sized listing.

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
