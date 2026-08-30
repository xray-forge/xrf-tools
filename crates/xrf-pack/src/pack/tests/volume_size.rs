//! The volume-size cap, end to end: that a produced file never exceeds it, that what an entry costs decides where it
//! lands, and that a cap the packer cannot keep is refused rather than quietly broken.
//!
//! The arithmetic is unit tested beside the types that own it, in `archive_volume_layout` and
//! `archive_descriptor_table`. These measure the files those decisions actually produce, which is the only place the
//! two can be caught disagreeing.

use xrf_archive::ArchiveProject;
use xrf_error::XrfError;

use crate::pack::archive_pack_config::{ArchivePackMode, VOLUME_SIZE_MAX};
use crate::pack::archive_packer::ArchivePacker;
use crate::pack::tests::fixtures::{
  CONFIG, assert_volumes_within, borrow_files, create_config, distinct_files, open, pack, read,
};

fn assert_pack_rejects_invalid_volume_size(scope: &str, max_volume_size: u64) {
  let (mut config, destination) = create_config(scope, &[("configs\\system.ltx", CONFIG)]);

  config.max_volume_size = max_volume_size;

  assert!(matches!(ArchivePacker::pack(&config), Err(XrfError::Invalid { .. })));
  assert!(!destination.exists(), "an invalid configuration creates no destination");
}

#[test]
fn splits_volumes_at_the_configured_size() {
  const MAX_VOLUME_SIZE: u64 = 8 * 1024;

  let files: Vec<(String, Vec<u8>)> = distinct_files(8, "dds", 4096);
  let (result, destination) = pack(
    "splits_volumes_at_the_configured_size",
    &borrow_files(&files),
    |config| config.max_volume_size = MAX_VOLUME_SIZE,
  );
  let project: ArchiveProject = open(&destination);

  assert!(result.volumes.len() > 1, "the set spans several volumes");
  assert_eq!(project.archives.len(), result.volumes.len());

  // A real set keeps its indices, starting at zero.
  assert_eq!(result.volumes[0].file_name().expect("name"), "packed.db0");
  assert_eq!(result.volumes[1].file_name().expect("name"), "packed.db1");

  // The cap is a maximum on the file, not on the position reached before the next entry: two 4096-byte payloads
  // sit inside 8192 bytes only until the chunk headers and the descriptor table land on top of them.
  assert_volumes_within(&result, MAX_VOLUME_SIZE);

  for (name, contents) in &files {
    assert_eq!(read(&project, name), *contents, "'{name}' survives the split");
  }
}

#[test]
fn places_an_entry_by_the_size_it_is_stored_at() {
  const MAX_VOLUME_SIZE: u64 = 4 * 1024;

  // Repetitive text, so the same tree needs several volumes stored and fits in one compressed.
  let files: Vec<(String, Vec<u8>)> = (0..8u8)
    .map(|index| {
      (
        format!("configs\\generated_{index}.ltx"),
        format!("[section_{index}]\n{}", "value = 1\n".repeat(150)).into_bytes(),
      )
    })
    .collect();
  let borrowed: Vec<(&str, &[u8])> = borrow_files(&files);
  let scope: &str = "places_an_entry_by_the_size_it_is_stored_at";

  let (compressed, compressed_destination) = pack(&format!("{scope}/compress"), &borrowed, |config| {
    config.max_volume_size = MAX_VOLUME_SIZE;
  });
  let (stored, _) = pack(&format!("{scope}/store"), &borrowed, |config| {
    config.max_volume_size = MAX_VOLUME_SIZE;
    config.mode = ArchivePackMode::Store;
  });

  assert_eq!(compressed.files_compressed, files.len(), "every entry shrank");
  assert!(
    compressed.volumes.len() < stored.volumes.len(),
    "a placement reading the source size would split these {} compressed volume(s) like the {} stored one(s)",
    compressed.volumes.len(),
    stored.volumes.len()
  );

  assert_volumes_within(&compressed, MAX_VOLUME_SIZE);
  assert_volumes_within(&stored, MAX_VOLUME_SIZE);

  let project: ArchiveProject = open(&compressed_destination);

  for (name, contents) in &files {
    assert_eq!(read(&project, name), *contents, "'{name}' survives the split");
  }
}

#[test]
fn keeps_a_cap_its_descriptors_and_header_nearly_fill() {
  const MAX_VOLUME_SIZE: u64 = 512;

  // Long names and tiny payloads, so the descriptor table rather than the data decides where a volume ends.
  let files: Vec<(String, Vec<u8>)> = (0..12u8)
    .map(|index| {
      (
        format!("textures\\a_deliberately_long_entry_name_{index}.dds"),
        vec![b'a' + index; 4],
      )
    })
    .collect();
  let (result, destination) = pack(
    "keeps_a_cap_its_descriptors_and_header_nearly_fill",
    &borrow_files(&files),
    |config| config.max_volume_size = MAX_VOLUME_SIZE,
  );
  let project: ArchiveProject = open(&destination);

  assert!(result.volumes.len() > 1, "the table alone spans several volumes");
  assert_volumes_within(&result, MAX_VOLUME_SIZE);

  for (name, contents) in &files {
    assert_eq!(read(&project, name), *contents, "'{name}' survives the split");
  }
}

#[test]
fn rejects_a_zero_volume_size_before_writing() {
  assert_pack_rejects_invalid_volume_size("rejects_a_zero_volume_size_before_writing", 0);
}

#[test]
fn rejects_an_oversized_volume_before_writing() {
  assert_pack_rejects_invalid_volume_size("rejects_an_oversized_volume_before_writing", VOLUME_SIZE_MAX + 1);
}

#[test]
fn refuses_a_cap_no_volume_could_open_within() {
  // Smaller than the header chunk and the two chunk headers every volume repeats, so no split can help.
  assert_pack_rejects_invalid_volume_size("refuses_a_cap_no_volume_could_open_within", 64);
}

#[test]
fn refuses_an_entry_no_volume_of_that_size_could_hold() {
  let (mut config, _) = create_config(
    "refuses_an_entry_no_volume_of_that_size_could_hold",
    &[("textures\\wall.dds", &[0x44; 4096])],
  );

  // Room for the chunks every volume carries, and none for this entry. Isolating it would publish a volume eight
  // times the advertised cap, so the pack is refused instead.
  config.max_volume_size = 512;

  assert!(matches!(ArchivePacker::pack(&config), Err(XrfError::Invalid { .. })));
}
