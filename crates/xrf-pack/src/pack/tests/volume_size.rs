//! The volume-size cap, end to end: that a produced file never exceeds it, that what an entry costs decides where it
//! lands, and that a cap the packer cannot keep is refused rather than quietly broken.
//!
//! The arithmetic is unit tested beside the types that own it, in `archive_volume_layout` and
//! `archive_descriptor_table`. These measure the files those decisions actually produce, which is the only place the
//! two can be caught disagreeing.

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;

use crate::pack::ArchivePacker;
use crate::pack::config::{ArchivePackMode, VOLUME_SIZE_MAX};
use crate::pack::tests::fixtures::{
  CONFIG, assert_one_payload, assert_volumes_within, borrow_files, create_config, descriptor, distinct_files, open,
  pack, read,
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
        format!("textures\\a_intentionally_long_entry_name_{index}.dds"),
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

#[test]
fn an_alias_stays_inside_the_volume_holding_its_payload() {
  const MAX_VOLUME_SIZE: u64 = 12 * 1024;
  const SHARED: &[u8] = &[b'x'; 4096];

  // Two payloads of this size fit one volume and three do not. Named so the walk writes them in this order.
  let files: [(&str, &[u8]); 5] = [
    ("textures\\a.dds", SHARED),
    ("textures\\b.dds", SHARED),
    ("textures\\c.dds", &[b'y'; 4096]),
    ("textures\\d.dds", &[b'z'; 4096]),
    ("textures\\e.dds", SHARED),
  ];
  let (result, destination) = pack(
    "an_alias_stays_inside_the_volume_holding_its_payload",
    &files,
    |config| config.max_volume_size = MAX_VOLUME_SIZE,
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.volumes.len(), 2);
  assert_volumes_within(&result, MAX_VOLUME_SIZE);

  // Inside one volume an identical file is a row pointing at the earlier payload.
  let shared: &ArchiveFileDescriptor = assert_one_payload(&project, &["textures\\a.dds", "textures\\b.dds"]);

  assert_eq!(shared.volume, 0);

  // Across volumes it is a payload again: an offset addresses one file, so the table forgets the earlier copy.
  let again: &ArchiveFileDescriptor = descriptor(&project, "textures\\e.dds");

  assert_eq!(again.volume, 1, "the third copy lands in the volume the split opened");
  assert_eq!(
    result.files_aliased, 1,
    "only the copy sharing a volume with its twin is an alias"
  );
  assert_eq!(result.files_stored, 4);

  for (name, contents) in files {
    assert_eq!(read(&project, name), contents, "'{name}' survives the split");
  }
}

#[test]
fn an_alias_whose_row_does_not_fit_is_written_again_in_the_next_volume() {
  const MAX_VOLUME_SIZE: u64 = 512;

  // Identical tiny payloads under long names: bytes are written once per volume and every later copy costs a row, so
  // it is the descriptor table that fills the cap. A row that no longer fits cannot point at a payload in a volume it
  // is not in, so that copy is written again as the first payload of the volume the split opens.
  let files: Vec<(String, Vec<u8>)> = (0..12u8)
    .map(|index| {
      (
        format!("textures\\a_intentionally_long_entry_name_{index}.dds"),
        vec![b'a'; 4],
      )
    })
    .collect();
  let (result, destination) = pack(
    "an_alias_whose_row_does_not_fit_is_written_again_in_the_next_volume",
    &borrow_files(&files),
    |config| config.max_volume_size = MAX_VOLUME_SIZE,
  );
  let project: ArchiveProject = open(&destination);

  assert!(result.volumes.len() > 1, "the rows alone span several volumes");
  assert_volumes_within(&result, MAX_VOLUME_SIZE);

  // Exactly one payload per volume: the first entry the volume takes, which every other entry in it aliases.
  assert_eq!(result.files_stored, result.volumes.len());
  assert_eq!(result.files_aliased, files.len() - result.volumes.len());

  for volume in 0..result.volumes.len() {
    let volume: u32 = u32::try_from(volume).expect("volume index");
    let names: Vec<&str> = files
      .iter()
      .map(|(name, _)| name.as_str())
      .filter(|name| descriptor(&project, name).volume == volume)
      .collect();

    assert!(!names.is_empty(), "volume {volume} holds entries");
    assert_one_payload(&project, &names);
  }

  for (name, contents) in &files {
    assert_eq!(read(&project, name), *contents, "'{name}' survives the split");
  }
}
