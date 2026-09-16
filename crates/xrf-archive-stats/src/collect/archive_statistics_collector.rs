use std::collections::HashMap;

use xrf_extension::XrayExtension;

use crate::collect::archive_statistics_entry::ArchiveStatisticsEntry;
use crate::report::archive_compression::ArchiveCompression;
use crate::report::archive_extension_usage::ArchiveExtensionUsage;
use crate::report::archive_folder_usage::ArchiveFolderUsage;
use crate::report::archive_largest_entry::ArchiveLargestEntry;
use crate::report::archive_measure::ArchiveMeasure;
use crate::report::archive_origins::ArchiveOrigins;
use crate::report::archive_overview::ArchiveOverview;
use crate::report::archive_size_band::ArchiveSizeBand;
use crate::report::archive_statistics::ArchiveStatistics;

/// How many entries the largest-files part of the report names.
const LARGEST_ENTRIES: usize = 20;

/// One group of entries keyed by something read off a name, with the stored size only while every member had one.
#[derive(Default)]
struct ArchiveGroup {
  measure: ArchiveMeasure,
  size_compressed: Option<u64>,
  is_measured: bool,
}

impl ArchiveGroup {
  fn add(&mut self, size_real: u64, size_compressed: Option<u64>) {
    self.size_compressed = match (self.is_measured, self.size_compressed, size_compressed) {
      (false, _, stored) => stored,
      (true, Some(total), Some(stored)) => Some(total + stored),
      _ => None,
    };

    self.is_measured = true;
    self.measure.add(size_real);
  }
}

/// Everything one walk of a subject's entries accumulates.
#[derive(Default)]
pub(crate) struct ArchiveStatisticsCollector {
  extensions: HashMap<Option<String>, ArchiveGroup>,
  folders: HashMap<Option<String>, ArchiveGroup>,
  bands: Vec<ArchiveMeasure>,
  /// Every file's unpacked size, kept sorted at the end so the median needs no second walk.
  sizes: Vec<u64>,
  largest: Vec<ArchiveLargestEntry>,
  total: ArchiveMeasure,
  directories: u64,
  empty_files: u64,
  size_compressed: u64,
  stored_uncompressed: u64,
  records_compression: bool,
}

impl ArchiveStatisticsCollector {
  /// Accumulates every entry of a subject.
  pub(crate) fn collect<'a, E: ArchiveStatisticsEntry + 'a>(entries: impl Iterator<Item = &'a E>) -> Self {
    let mut collector: Self = Self {
      bands: vec![ArchiveMeasure::default(); ArchiveSizeBand::FLOORS.len()],
      ..Self::default()
    };

    for entry in entries {
      collector.add(entry);
    }

    collector
  }

  /// Turns the walk into the report, with the source count its own kind of subject supplies.
  ///
  /// Leaves `volumes` and `origins` empty: neither falls out of a walk of entries, and each constructor fills its own.
  pub(crate) fn into_report(mut self, sources: u64) -> ArchiveStatistics {
    self.sizes.sort_unstable();

    ArchiveStatistics {
      // Read before the walk is taken apart: these borrow what the moves below consume.
      overview: self.to_overview(sources),
      compression: self.to_compression(),
      sizes: self.to_size_bands(),
      extensions: Self::into_extension_usage(self.extensions),
      folders: Self::into_folder_usage(self.folders),
      largest: self.largest,
      // Neither is knowable from a walk of entries alone, so each constructor supplies its own.
      volumes: None,
      origins: ArchiveOrigins::default(),
    }
  }

  fn add<E: ArchiveStatisticsEntry>(&mut self, entry: &E) {
    if entry.is_directory() {
      self.directories += 1;

      return;
    }

    let name: &str = entry.get_name();
    let size: u64 = entry.get_size_real();
    let stored: Option<u64> = entry.get_size_compressed();

    self.total.add(size);
    self.sizes.push(size);
    self.bands[ArchiveSizeBand::index_of(size)].add(size);

    if size == 0 {
      self.empty_files += 1;
    }

    if let Some(stored) = stored {
      self.records_compression = true;
      self.size_compressed += stored;

      // Equal sizes is how the format says "stored uncompressed". Not a defect: Anomaly stores every `geom`, `cform`
      // and `ai` payload this way.
      if stored == size {
        self.stored_uncompressed += 1;
      }
    }

    self
      .extensions
      .entry(Self::extension_of(name).map(str::to_string))
      .or_default()
      .add(size, stored);

    self
      .folders
      .entry(Self::folder_of(name).map(str::to_string))
      .or_default()
      .add(size, stored);

    self.keep_if_largest(name, size, stored);
  }

  /// Keeps the running largest entries, bounded, rather than sorting every name once the walk is over.
  fn keep_if_largest(&mut self, name: &str, size_real: u64, size_compressed: Option<u64>) {
    if self.largest.len() == LARGEST_ENTRIES
      && let Some(smallest) = self.largest.last()
      && size_real <= smallest.size_real
    {
      return;
    }

    let at: usize = self.largest.partition_point(|held| held.size_real > size_real);

    self.largest.insert(
      at,
      ArchiveLargestEntry {
        name: name.to_string(),
        size_real,
        size_compressed,
      },
    );
    self.largest.truncate(LARGEST_ENTRIES);
  }

  /// Reads the overview off a walk whose sizes are already sorted.
  fn to_overview(&self, sources: u64) -> ArchiveOverview {
    ArchiveOverview {
      total: self.total,
      sources,
      directories: self.directories,
      empty_files: self.empty_files,
      largest_file: self.sizes.last().copied().unwrap_or_default(),
      mean_file: self.total.size_real.checked_div(self.total.files).unwrap_or_default(),
      median_file: self.sizes.get(self.sizes.len() / 2).copied().unwrap_or_default(),
    }
  }

  fn to_size_bands(&self) -> Vec<ArchiveSizeBand> {
    self
      .bands
      .iter()
      .enumerate()
      .filter(|(_, measure)| !measure.is_empty())
      .map(|(band, measure)| ArchiveSizeBand {
        from: ArchiveSizeBand::FLOORS[band],
        to: ArchiveSizeBand::ceiling(band),
        measure: *measure,
      })
      .collect()
  }

  fn to_compression(&self) -> Option<ArchiveCompression> {
    self.records_compression.then_some(ArchiveCompression {
      size_compressed: self.size_compressed,
      size_real: self.total.size_real,
      stored_uncompressed: self.stored_uncompressed,
    })
  }

  /// Heaviest first, with the key breaking ties so two readings of one subject order equal groups the same way.
  fn into_extension_usage(groups: HashMap<Option<String>, ArchiveGroup>) -> Vec<ArchiveExtensionUsage> {
    let mut rows: Vec<ArchiveExtensionUsage> = groups
      .into_iter()
      .map(|(extension, group)| ArchiveExtensionUsage {
        is_declared: extension
          .as_deref()
          .is_some_and(|spelling| XrayExtension::parse(spelling).is_some()),
        extension,
        measure: group.measure,
        size_compressed: group.size_compressed,
      })
      .collect();

    rows.sort_by(|first, second| {
      second
        .measure
        .size_real
        .cmp(&first.measure.size_real)
        .then_with(|| first.extension.cmp(&second.extension))
    });

    rows
  }

  fn into_folder_usage(groups: HashMap<Option<String>, ArchiveGroup>) -> Vec<ArchiveFolderUsage> {
    let mut rows: Vec<ArchiveFolderUsage> = groups
      .into_iter()
      .map(|(folder, group)| ArchiveFolderUsage {
        folder,
        measure: group.measure,
        size_compressed: group.size_compressed,
      })
      .collect();

    rows.sort_by(|first, second| {
      second
        .measure
        .size_real
        .cmp(&first.measure.size_real)
        .then_with(|| first.folder.cmp(&second.folder))
    });

    rows
  }

  /// The extension an engine path's last segment spells, lower-cased, or `None` when it has none.
  fn extension_of(name: &str) -> Option<&str> {
    let base: &str = name.rsplit(['\\', '/']).next()?;
    let dot: usize = base.rfind('.')?;

    (dot > 0).then(|| &base[dot + 1..]).filter(|it| !it.is_empty())
  }

  /// The first segment of an engine path, or `None` for a file sitting at the root.
  fn folder_of(name: &str) -> Option<&str> {
    let (folder, _) = name.split_once(['\\', '/'])?;

    (!folder.is_empty()).then_some(folder)
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveStatisticsCollector;

  #[test]
  fn an_extension_is_read_off_the_last_segment() {
    assert_eq!(
      ArchiveStatisticsCollector::extension_of("textures\\wpn\\ak74.dds"),
      Some("dds")
    );
    assert_eq!(ArchiveStatisticsCollector::extension_of("ak74.dds"), Some("dds"));
    assert_eq!(
      ArchiveStatisticsCollector::extension_of("levels\\l01_escape\\level.geom"),
      Some("geom"),
      "a dotted directory above it does not decide the answer"
    );
  }

  #[test]
  fn a_name_without_an_extension_reports_none() {
    assert_eq!(ArchiveStatisticsCollector::extension_of("configs\\readme"), None);
    assert_eq!(
      ArchiveStatisticsCollector::extension_of("configs\\.gitignore"),
      None,
      "a leading dot is a name"
    );
    assert_eq!(
      ArchiveStatisticsCollector::extension_of("configs\\trailing."),
      None,
      "and a trailing dot spells nothing"
    );
  }

  #[test]
  fn a_folder_is_the_first_segment() {
    assert_eq!(
      ArchiveStatisticsCollector::folder_of("textures\\wpn\\ak74.dds"),
      Some("textures")
    );
    assert_eq!(
      ArchiveStatisticsCollector::folder_of("system.ltx"),
      None,
      "a file at the root sits in no folder"
    );
  }
}
