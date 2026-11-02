use std::collections::HashMap;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_vfs::XrayAssetContainer;

use crate::collect::archive_statistics_entry::ArchiveStatisticsEntry;
use crate::collect::archive_world_statistics_entry::ArchiveWorldStatisticsEntry;
use crate::report::archive_measure::ArchiveMeasure;
use crate::report::archive_origins::ArchiveOrigins;
use crate::report::archive_source_usage::ArchiveSourceUsage;

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
  /// Where a mounted world's winning entries come from, and the copies each one hides, with `sources` in search
  /// priority order.
  pub(crate) fn collect_world<E: ArchiveWorldStatisticsEntry>(entries: &[E], sources: &[String]) -> ArchiveOrigins {
    let mut collector: Self = Self::default();

    for entry in entries {
      let container: &XrayAssetContainer = entry.get_container();

      collector.add_winner(
        Self::source_of(container),
        Self::is_loose(container),
        entry.get_size_real(),
      );

      for (container, size) in entry.list_shadowed() {
        collector.add_hidden(Self::source_of(container), Self::is_loose(container), size);
      }
    }

    collector.into_origins(sources)
  }

  /// Where a volume set's entries come from, and what its own merge hides.
  pub(crate) fn collect_volumes(project: &ArchiveProject) -> ArchiveOrigins {
    let mut collector: Self = Self::default();

    for descriptor in project.files.values().filter(|descriptor| !descriptor.is_directory()) {
      collector.add_winner(Self::volume_of(project, descriptor), false, descriptor.get_size_real());
    }

    for descriptor in &project.shadowed {
      collector.add_hidden(Self::volume_of(project, descriptor), false, descriptor.get_size_real());
    }

    // Reverse merge order, because a later volume wins the name table and so is the one a lookup reaches first.
    let sources: Vec<String> = project
      .archives
      .iter()
      .rev()
      .map(|volume| volume.path.display().to_string())
      .collect();

    collector.into_origins(&sources)
  }

  /// Counts one entry a lookup reaches against the source serving it.
  fn add_winner(&mut self, source: String, is_loose: bool, size: u64) {
    if is_loose {
      self.origins.loose.add(size);
    } else {
      self.origins.archived.add(size);
    }

    self.tally(source, is_loose).wins.add(size);
  }

  /// Counts one copy no lookup reaches against the source holding it.
  fn add_hidden(&mut self, source: String, is_loose: bool, size: u64) {
    self.origins.hidden.add(size);
    self.tally(source, is_loose).hides.add(size);
  }

  /// The running tally for a source, created on first sight.
  fn tally(&mut self, source: String, is_loose: bool) -> &mut SourceTally {
    let tally: &mut SourceTally = self.tallies.entry(source).or_default();

    tally.is_loose = is_loose;

    tally
  }

  /// Turns the walk into the report, with one entry per source in the order they are searched.
  fn into_origins(mut self, sources: &[String]) -> ArchiveOrigins {
    let mut named: Vec<String> = sources.to_vec();
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

  /// The volume file one entry sits in, which is the grain a copy is attributed to.
  fn volume_of(project: &ArchiveProject, descriptor: &ArchiveFileDescriptor) -> String {
    project.archives.get(descriptor.volume as usize).map_or_else(
      || project.root.display().to_string(),
      |volume| volume.path.display().to_string(),
    )
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
