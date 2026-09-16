use std::collections::HashMap;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_vfs::XrayAssetContainer;

use crate::collect::archive_statistics_entry::ArchiveWorldStatisticsEntry;
use crate::report::archive_measure::ArchiveMeasure;
use crate::report::archive_origins::{ArchiveOrigins, ArchiveSourceUsage};

/// What one source has won and lost so far, while the walk is still running.
#[derive(Default)]
struct SourceTally {
  is_loose: bool,
  wins: ArchiveMeasure,
  hides: ArchiveMeasure,
}

/// Where a subject's files come from, and what its own ordering hides.
#[derive(Default)]
pub(crate) struct ArchiveOriginsCollector {
  origins: ArchiveOrigins,
  tallies: HashMap<String, SourceTally>,
}

impl ArchiveOriginsCollector {
  /// Accumulates every winning entry and the copies each one hides.
  pub(crate) fn collect<E: ArchiveWorldStatisticsEntry>(entries: &[E], mounts: &[String]) -> ArchiveOrigins {
    let mut collector: Self = Self::default();

    for entry in entries {
      collector.add(entry);
    }

    collector.into_origins(mounts)
  }

  fn add<E: ArchiveWorldStatisticsEntry>(&mut self, entry: &E) {
    let size: u64 = entry.get_size_real();
    let container: &XrayAssetContainer = entry.get_container();

    if Self::is_loose(container) {
      self.origins.loose.add(size);
    } else {
      self.origins.archived.add(size);
    }

    self.tally(container).wins.add(size);

    for (container, size) in entry.list_shadowed() {
      self.origins.hidden.add(size);
      self.tally(container).hides.add(size);
    }
  }

  /// Where a volume set's entries come from and what its own merge hides.
  pub(crate) fn collect_volumes(project: &ArchiveProject, sources: &[String]) -> ArchiveOrigins {
    let mut collector: Self = Self::default();

    for descriptor in project.files.values() {
      if descriptor.is_directory {
        continue;
      }

      let size: u64 = u64::from(descriptor.size_real);

      collector.origins.archived.add(size);
      collector
        .tally_of(Self::volume_of(project, descriptor), false)
        .wins
        .add(size);
    }

    for descriptor in &project.shadowed {
      let size: u64 = u64::from(descriptor.size_real);

      collector.origins.hidden.add(size);
      collector
        .tally_of(Self::volume_of(project, descriptor), false)
        .hides
        .add(size);
    }

    collector.into_origins(sources)
  }

  /// The running tally for whatever source a container names, created on first sight.
  fn tally(&mut self, container: &XrayAssetContainer) -> &mut SourceTally {
    self.tally_of(Self::source_of(container), Self::is_loose(container))
  }

  /// The running tally for a source named directly, created on first sight.
  fn tally_of(&mut self, source: String, is_loose: bool) -> &mut SourceTally {
    let tally: &mut SourceTally = self.tallies.entry(source).or_default();

    tally.is_loose = is_loose;

    tally
  }

  /// The volume file one entry sits in, which is the grain a copy is attributed to.
  fn volume_of(project: &ArchiveProject, descriptor: &ArchiveFileDescriptor) -> String {
    project.archives.get(descriptor.volume as usize).map_or_else(
      || project.root.display().to_string(),
      |volume| volume.path.display().to_string(),
    )
  }

  /// Turns the walk into the report, with one entry per source, mount order first.
  fn into_origins(mut self, mounts: &[String]) -> ArchiveOrigins {
    let mut named: Vec<String> = mounts.to_vec();
    let mut unnamed: Vec<String> = self
      .tallies
      .keys()
      .filter(|source| !named.contains(source))
      .cloned()
      .collect();

    // Sorted, because a hash map hands them over in whatever order it pleases and a report has to be reproducible.
    unnamed.sort();
    named.append(&mut unnamed);

    self.origins.sources = named
      .into_iter()
      .map(|source| {
        let tally: SourceTally = self.tallies.remove(&source).unwrap_or_default();

        ArchiveSourceUsage {
          source,
          is_loose: tally.is_loose,
          wins: tally.wins,
          hides: tally.hides,
        }
      })
      .collect();

    self.origins
  }

  /// The source a container names, as the world's own mount list spells it.
  fn source_of(container: &XrayAssetContainer) -> String {
    match container {
      XrayAssetContainer::Directory { root, .. } => root.display().to_string(),
      XrayAssetContainer::Archive { path } => path.display().to_string(),
    }
  }

  fn is_loose(container: &XrayAssetContainer) -> bool {
    matches!(container, XrayAssetContainer::Directory { .. })
  }
}
