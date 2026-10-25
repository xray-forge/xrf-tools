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
use crate::report::archive_volume_summary::ArchiveVolumeSummary;

/// How many entries the largest-files part of the report names.
const LARGEST_ENTRIES: usize = 20;

/// One group of entries keyed by something read off a name, with the stored size only while every member had one.
#[derive(Default)]
struct ArchiveGroup {
  measure: ArchiveMeasure,
  size_compressed: Option<u64>,
}

impl ArchiveGroup {
  fn add(&mut self, size_real: u64, size_compressed: Option<u64>) {
    self.size_compressed = match (self.measure.is_empty(), self.size_compressed, size_compressed) {
      (true, _, stored) => stored,
      (false, Some(total), Some(stored)) => Some(total + stored),
      _ => None,
    };

    self.measure.add(size_real);
  }
}

/// Everything one walk of a subject's entries accumulates.
#[derive(Default)]
pub(crate) struct ArchiveStatisticsCollector {
  extensions: HashMap<Option<String>, ArchiveGroup>,
  folders: HashMap<Option<String>, ArchiveGroup>,
  bands: [ArchiveMeasure; ArchiveSizeBand::FLOORS.len()],
  /// Every file's unpacked size, kept sorted at the end so the median needs no second walk.
  sizes: Vec<u64>,
  largest: Vec<ArchiveLargestEntry>,
  total: ArchiveMeasure,
  directories: u64,
  empty_files: u64,
  /// Stored bytes of every file that supplied a stored size, or `None` while none has.
  size_compressed: Option<u64>,
  stored_uncompressed: u64,
}

impl ArchiveStatisticsCollector {
  /// Accumulates every entry of a subject.
  pub(crate) fn collect<'a, E: ArchiveStatisticsEntry + 'a>(entries: impl Iterator<Item = &'a E>) -> Self {
    let mut collector: Self = Self::default();

    for entry in entries {
      collector.add(entry);
    }

    collector
  }

  /// Turns the walk into the report, completed by what only the subject itself knows: how many sources it counts,
  /// its volumes, and where its files come from.
  pub(crate) fn into_report(
    mut self,
    sources: u64,
    volumes: Option<Vec<ArchiveVolumeSummary>>,
    origins: ArchiveOrigins,
  ) -> ArchiveStatistics {
    self.sizes.sort_unstable();

    ArchiveStatistics {
      // Read before the walk is taken apart: these borrow what the moves below consume.
      overview: self.to_overview(sources),
      compression: self.to_compression(),
      sizes: self.to_size_bands(),
      extensions: Self::into_extension_usage(self.extensions),
      folders: Self::into_folder_usage(self.folders),
      largest: self.largest,
      volumes,
      origins,
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
      self.size_compressed = Some(self.size_compressed.unwrap_or_default() + stored);

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
    self.size_compressed.map(|size_compressed| ArchiveCompression {
      size_compressed,
      size_real: self.total.size_real,
      stored_uncompressed: self.stored_uncompressed,
    })
  }

  fn into_extension_usage(groups: HashMap<Option<String>, ArchiveGroup>) -> Vec<ArchiveExtensionUsage> {
    Self::into_heaviest_first(groups)
      .map(|(extension, group)| ArchiveExtensionUsage {
        is_declared: extension
          .as_deref()
          .is_some_and(|spelling| XrayExtension::parse(spelling).is_some()),
        extension,
        measure: group.measure,
        size_compressed: group.size_compressed,
      })
      .collect()
  }

  fn into_folder_usage(groups: HashMap<Option<String>, ArchiveGroup>) -> Vec<ArchiveFolderUsage> {
    Self::into_heaviest_first(groups)
      .map(|(folder, group)| ArchiveFolderUsage {
        folder,
        measure: group.measure,
        size_compressed: group.size_compressed,
      })
      .collect()
  }

  /// Heaviest first, with the key breaking ties so two readings of one subject order equal groups the same way.
  fn into_heaviest_first(
    groups: HashMap<Option<String>, ArchiveGroup>,
  ) -> impl Iterator<Item = (Option<String>, ArchiveGroup)> {
    let mut groups: Vec<(Option<String>, ArchiveGroup)> = groups.into_iter().collect();

    groups.sort_by(|(first_key, first), (second_key, second)| {
      second
        .measure
        .size_real
        .cmp(&first.measure.size_real)
        .then_with(|| first_key.cmp(second_key))
    });

    groups.into_iter()
  }

  /// The extension an engine path's last segment spells, as it spells it, or `None` when it has none.
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
  use super::{ArchiveGroup, ArchiveStatisticsCollector, LARGEST_ENTRIES};

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

  #[test]
  fn a_group_keeps_a_stored_size_only_while_every_member_supplies_one() {
    let mut group: ArchiveGroup = ArchiveGroup::default();

    group.add(100, Some(40));
    group.add(200, Some(60));

    assert_eq!(group.size_compressed, Some(100));

    group.add(300, None);

    assert_eq!(
      group.size_compressed, None,
      "a partial sum would pass for the whole group's"
    );

    group.add(400, Some(80));

    assert_eq!(group.size_compressed, None, "and a later member cannot bring it back");
    assert_eq!(group.measure.files, 4);
  }

  #[test]
  fn the_largest_entries_stay_bounded_and_ordered() {
    let mut collector: ArchiveStatisticsCollector = ArchiveStatisticsCollector::default();

    for size in 0..(LARGEST_ENTRIES as u64 + 5) {
      collector.keep_if_largest(&format!("file_{size}"), size, None);
    }

    let sizes: Vec<u64> = collector.largest.iter().map(|entry| entry.size_real).collect();

    assert_eq!(sizes.len(), LARGEST_ENTRIES);
    assert_eq!(sizes.first(), Some(&(LARGEST_ENTRIES as u64 + 4)), "largest first");
    assert!(
      sizes.windows(2).all(|pair| pair[0] > pair[1]),
      "and descending throughout"
    );
  }
}
