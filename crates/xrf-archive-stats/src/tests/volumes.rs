use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveReadPolicy};

use crate::{ArchiveExtensionUsage, ArchiveStatistics};

fn descriptor(name: &str, size_real: u32, size_compressed: u32, is_directory: bool) -> ArchiveFileDescriptor {
  ArchiveFileDescriptor {
    crc: 0,
    is_directory,
    name: Arc::from(name),
    offset: 0,
    size_compressed,
    size_real,
    volume: 0,
  }
}

/// A volume set holding one of each thing a breakdown has to treat differently.
fn project() -> ArchiveProject {
  let files: Vec<ArchiveFileDescriptor> = vec![
    descriptor("textures\\wpn\\ak74.dds", 4096, 2048, false),
    descriptor("textures\\wpn\\pm.dds", 2048, 1024, false),
    descriptor("configs\\system.ltx", 1000, 100, false),
    descriptor("sounds\\ambient.ogg", 8192, 8192, false),
    descriptor("readme", 0, 0, false),
    descriptor("meshes\\actor.som", 512, 512, false),
    descriptor("textures\\", 0, 0, true),
  ];

  ArchiveProject {
    archives: vec![ArchiveDescriptor {
      created_at: None,
      modified_at: Some(1_700_000_000_000),
      entries: 7,
      output_root_path: PathBuf::from("gamedata/"),
      path: PathBuf::from("C:/game/db/textures.db0"),
      size_compressed: 11_876,
      size_real: 15_848,
    }],
    files: files
      .into_iter()
      .map(|descriptor| (Arc::clone(&descriptor.name), descriptor))
      .collect::<HashMap<Arc<str>, ArchiveFileDescriptor>>(),
    read_policy: ArchiveReadPolicy::default(),
    root: PathBuf::from("C:/game/db"),
    size_real: 15_848,
  }
}

fn row<'a>(rows: &'a [ArchiveExtensionUsage], extension: Option<&str>) -> &'a ArchiveExtensionUsage {
  rows
    .iter()
    .find(|row| row.extension.as_deref() == extension)
    .expect("the extension is reported")
}

#[test]
fn an_overview_counts_files_and_leaves_directories_out_of_the_bytes() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  assert_eq!(statistics.overview.total.files, 6, "the directory entry is not a file");
  assert_eq!(statistics.overview.directories, 1);
  assert_eq!(
    statistics.overview.total.size_real,
    4096 + 2048 + 1000 + 8192 + 512,
    "and holds no bytes to count"
  );
  assert_eq!(statistics.overview.sources, 1, "one volume answered");
  assert_eq!(statistics.overview.empty_files, 1, "the zero-length file is named");
  assert_eq!(statistics.overview.largest_file, 8192);
}

#[test]
fn an_undeclared_spelling_is_reported_as_itself() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  // The point of the section: `som` is a real X-Ray format the extension vocabulary does not declare, and folding it
  // into an `Other` row would hide the one row worth acting on.
  let som: &ArchiveExtensionUsage = row(&statistics.extensions, Some("som"));

  assert!(!som.is_declared, "the vocabulary does not declare it");
  assert_eq!(som.measure.files, 1);

  assert!(row(&statistics.extensions, Some("dds")).is_declared);
  assert!(row(&statistics.extensions, Some("ltx")).is_declared);
}

#[test]
fn a_name_without_an_extension_gets_its_own_row() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());
  let none: &ArchiveExtensionUsage = row(&statistics.extensions, None);

  assert_eq!(none.measure.files, 1);
  assert!(
    !none.is_declared,
    "nothing to recognize is not the same as something unrecognized"
  );
}

#[test]
fn extensions_are_ordered_by_bytes_and_sum_back_to_the_total() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  assert_eq!(
    statistics
      .extensions
      .iter()
      .map(|row| row.extension.as_deref())
      .collect::<Vec<Option<&str>>>(),
    vec![Some("ogg"), Some("dds"), Some("ltx"), Some("som"), None],
    "heaviest first: one ogg outweighs two dds, which is the ordering a count column would invert"
  );

  assert_eq!(
    statistics
      .extensions
      .iter()
      .map(|row| row.measure.size_real)
      .sum::<u64>(),
    statistics.overview.total.size_real,
    "every byte is in exactly one row"
  );
  assert_eq!(
    statistics.extensions.iter().map(|row| row.measure.files).sum::<u64>(),
    statistics.overview.total.files
  );
}

#[test]
fn folders_group_by_the_first_segment_and_a_root_file_sits_in_none() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  let folders: Vec<Option<&str>> = statistics.folders.iter().map(|row| row.folder.as_deref()).collect();

  assert_eq!(
    folders,
    vec![Some("sounds"), Some("textures"), Some("configs"), Some("meshes"), None]
  );
  assert_eq!(
    statistics.folders.iter().map(|row| row.measure.files).sum::<u64>(),
    statistics.overview.total.files,
    "every file is in exactly one folder row"
  );
}

#[test]
fn compression_reads_the_stored_sizes_the_format_records() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());
  let compression = statistics.compression.expect("a volume set records stored sizes");

  // The zero-length file contributes nothing to either side and is left out of the sum rather than added as zero.
  assert_eq!(compression.size_compressed, 2048 + 1024 + 100 + 8192 + 512);
  assert_eq!(compression.size_real, statistics.overview.total.size_real);
  assert_eq!(
    compression.stored_uncompressed, 3,
    "the ogg, the som and the empty file are each stored at their own size"
  );
}

#[test]
fn a_volume_row_reads_the_volume_descriptor_rather_than_recounting_the_merged_table() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());
  let volumes: Vec<crate::ArchiveVolumeSummary> = statistics.volumes.expect("a volume set has volumes");

  assert_eq!(volumes.len(), 1);
  assert_eq!(volumes[0].entries, 7, "including the directory entry the table holds");
  assert_eq!(volumes[0].modified_at, Some(1_700_000_000_000));
  assert_eq!(volumes[0].path, PathBuf::from("C:/game/db/textures.db0"));
}

#[test]
fn a_volume_set_cannot_answer_what_its_merge_folded_away() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  // Absent rather than empty: the name table keeps one entry per name, so the copies it merged away are gone before
  // anything could count them. An empty answer would claim nothing was shadowed.
  assert!(statistics.origins.is_none());
}

#[test]
fn largest_entries_are_ordered_and_addressed_by_engine_path() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  assert_eq!(
    statistics
      .largest
      .iter()
      .map(|entry| entry.name.as_str())
      .collect::<Vec<&str>>(),
    vec![
      "sounds\\ambient.ogg",
      "textures\\wpn\\ak74.dds",
      "textures\\wpn\\pm.dds",
      "configs\\system.ltx",
      "meshes\\actor.som",
      "readme"
    ]
  );
}

#[test]
fn size_bands_omit_what_nothing_fell_into_and_keep_empty_files_apart() {
  let statistics: ArchiveStatistics = ArchiveStatistics::of_volumes(&project());

  assert_eq!(
    statistics
      .sizes
      .first()
      .map(|bucket| (bucket.from, bucket.measure.files)),
    Some((0, 1)),
    "the zero-length file is its own band"
  );
  assert!(
    statistics.sizes.iter().all(|bucket| !bucket.measure.is_empty()),
    "a band nothing fell into is not reported"
  );
  assert_eq!(
    statistics.sizes.iter().map(|bucket| bucket.measure.files).sum::<u64>(),
    statistics.overview.total.files
  );
}
