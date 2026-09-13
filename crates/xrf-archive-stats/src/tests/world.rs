use std::path::PathBuf;

use xrf_vfs::XrayAssetContainer;

use crate::{
  ArchiveOrigins, ArchiveSourceUsage, ArchiveStatistics, ArchiveStatisticsEntry, ArchiveWorldStatisticsEntry,
};

const LOOSE_ROOT: &str = "C:/game/gamedata";
const VOLUME: &str = "C:/game/db/textures.db0";

/// Stands in for whatever a consumer already retains, which is the whole point of the trait seam.
struct Entry {
  name: String,
  size_real: u64,
  container: XrayAssetContainer,
  shadowed: Vec<(XrayAssetContainer, u64)>,
}

impl ArchiveStatisticsEntry for Entry {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_size_real(&self) -> u64 {
    self.size_real
  }
}

impl ArchiveWorldStatisticsEntry for Entry {
  fn get_container(&self) -> &XrayAssetContainer {
    &self.container
  }

  fn list_shadowed(&self) -> impl Iterator<Item = (&XrayAssetContainer, u64)> {
    self.shadowed.iter().map(|(container, size)| (container, *size))
  }
}

fn loose(relative_path: &str) -> XrayAssetContainer {
  XrayAssetContainer::Directory {
    root: PathBuf::from(LOOSE_ROOT),
    relative_path: PathBuf::from(relative_path),
  }
}

fn archived() -> XrayAssetContainer {
  XrayAssetContainer::Archive {
    path: PathBuf::from(VOLUME),
  }
}

/// A loose mod tree in front of a volume set: one overridden path, one loose-only, one archived-only.
fn world() -> Vec<Entry> {
  vec![
    Entry {
      name: String::from("configs\\system.ltx"),
      size_real: 1000,
      container: loose("configs\\system.ltx"),
      // Deliberately a different size from the winner: a rollup reporting the winner's bytes would pass otherwise.
      shadowed: vec![(archived(), 7000)],
    },
    Entry {
      name: String::from("configs\\only_loose.ltx"),
      size_real: 200,
      container: loose("configs\\only_loose.ltx"),
      shadowed: Vec::new(),
    },
    Entry {
      name: String::from("textures\\wpn\\ak74.dds"),
      size_real: 4096,
      container: archived(),
      shadowed: Vec::new(),
    },
  ]
}

fn mounts() -> Vec<String> {
  vec![String::from(LOOSE_ROOT), String::from(VOLUME)]
}

fn source<'a>(origins: &'a ArchiveOrigins, name: &str) -> &'a ArchiveSourceUsage {
  origins
    .sources
    .iter()
    .find(|row| row.source == name)
    .expect("the source is reported")
}

#[test]
fn a_world_counts_only_what_the_engine_would_load() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());

  assert_eq!(statistics.overview.total.files, 3);
  assert_eq!(
    statistics.overview.total.size_real,
    1000 + 200 + 4096,
    "the hidden copy is not part of what the engine loads"
  );
  assert_eq!(statistics.overview.sources, 2, "both mounts answered");
}

#[test]
fn a_world_records_no_stored_size_and_no_volumes() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());

  // Absent rather than zero: a loose file has no stored size, so a compression figure here would be invented.
  assert!(statistics.compression.is_none());
  assert!(statistics.volumes.is_none());
  assert!(
    statistics.extensions.iter().all(|row| row.size_compressed.is_none()),
    "and no group can claim one either"
  );
}

#[test]
fn origins_split_what_wins_by_where_it_is_read_from() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());
  let origins: ArchiveOrigins = statistics.origins.expect("a world knows where its files come from");

  assert_eq!(origins.loose.files, 2);
  assert_eq!(origins.loose.size_real, 1200);
  assert_eq!(origins.archived.files, 1);
  assert_eq!(origins.archived.size_real, 4096);
  assert_eq!(
    origins.loose.files + origins.archived.files,
    statistics.overview.total.files,
    "every winner is read from exactly one kind of place"
  );
}

#[test]
fn overrides_are_counted_apart_from_what_the_engine_loads() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());
  let origins: ArchiveOrigins = statistics.origins.expect("a world knows what it hides");

  assert_eq!(origins.hidden.files, 1);
  assert_eq!(
    origins.hidden.size_real, 7000,
    "the hidden copy is measured by the mount holding it, not by the winner standing in front of it"
  );
}

#[test]
fn a_source_row_says_both_what_it_wins_and_what_it_loses() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());
  let origins: ArchiveOrigins = statistics.origins.expect("a world has sources");

  let tree: &ArchiveSourceUsage = source(&origins, LOOSE_ROOT);

  assert!(tree.is_loose);
  assert_eq!(tree.wins.files, 2, "the mod tree answers two paths");
  assert_eq!(tree.hides.files, 0, "and nothing in front of it hides anything");

  let volume: &ArchiveSourceUsage = source(&origins, VOLUME);

  assert!(!volume.is_loose);
  assert_eq!(
    volume.wins.files, 1,
    "the volume still answers the path nothing overrides"
  );
  assert_eq!(volume.hides.files, 1, "and loses the one the tree overrode");
  assert_eq!(volume.hides.size_real, 7000);
}

#[test]
fn sources_are_listed_in_mount_order() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());
  let origins: ArchiveOrigins = statistics.origins.expect("a world has sources");

  assert_eq!(
    origins
      .sources
      .iter()
      .map(|row| row.source.as_str())
      .collect::<Vec<&str>>(),
    vec![LOOSE_ROOT, VOLUME],
    "priority order, so the row above is the one that wins"
  );
}

#[test]
fn a_mount_answering_nothing_is_still_reported() {
  let mut mounts: Vec<String> = mounts();

  mounts.push(String::from("C:/game/db/empty.db0"));

  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts);
  let origins: ArchiveOrigins = statistics.origins.expect("a world has sources");

  // A mount contributing nothing is a finding rather than an absence: it is usually a mod that did not load.
  let empty: &ArchiveSourceUsage = source(&origins, "C:/game/db/empty.db0");

  assert!(empty.wins.is_empty());
  assert!(empty.hides.is_empty());
}

#[test]
fn a_source_no_mount_named_is_still_reported_in_a_stable_place() {
  // A volume inside a mounted set names itself, so a container can name a source the mount list does not. Dropping it
  // would lose bytes the totals already counted, and leaving it in hash order would make the report unreproducible.
  let mounts: Vec<String> = vec![String::from(LOOSE_ROOT)];

  let first: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts);
  let second: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts);

  let names = |statistics: ArchiveStatistics| -> Vec<String> {
    statistics
      .origins
      .expect("a world has sources")
      .sources
      .into_iter()
      .map(|source| source.source)
      .collect()
  };

  assert_eq!(
    names(first),
    vec![String::from(LOOSE_ROOT), String::from(VOLUME)],
    "the named mount leads, and the volume that named itself follows it"
  );
  assert_eq!(names(second), vec![String::from(LOOSE_ROOT), String::from(VOLUME)]);
}

#[test]
fn every_winning_byte_is_attributed_to_exactly_one_source() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_world(&world(), &mounts());
  let origins: ArchiveOrigins = statistics.origins.expect("a world has sources");

  assert_eq!(
    origins.sources.iter().map(|source| source.wins.size_real).sum::<u64>(),
    statistics.overview.total.size_real,
    "the per-source rollup and the overview describe the same tree"
  );
  assert_eq!(
    origins.sources.iter().map(|source| source.hides.size_real).sum::<u64>(),
    origins.hidden.size_real,
    "and what the sources lose sums to what the world hides"
  );
}
