//! What one entry costs the volume it lands in: compressed, stored, reverted, empty, or a row aliasing a payload
//! already written.
//!
//! Read through the descriptors as well as the counts, because a count proves a decision was made and the descriptor
//! proves the volume records it. Where an entry lands is `volume_size`.

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};

use crate::pack::config::ArchivePackMode;
use crate::pack::tests::fixtures::{BINARY, CONFIG, assert_one_payload, descriptor, open, pack, read};

#[test]
fn compresses_configuration_and_stores_everything_else() {
  let (result, destination) = pack(
    "compresses_configuration_and_stores_everything_else",
    &[("configs\\system.ltx", CONFIG), ("textures\\wall.dds", BINARY)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_compressed, 1, "only the config is worth compressing");
  assert_eq!(result.files_stored, 1);

  let config: &ArchiveFileDescriptor = project.files.get("configs\\system.ltx").expect("config entry");
  let texture: &ArchiveFileDescriptor = project.files.get("textures\\wall.dds").expect("texture entry");

  assert!(config.size_compressed < config.size_real, "the config shrank");
  assert_eq!(
    texture.size_compressed, texture.size_real,
    "a stored entry declares equal sizes, which is how the reader tells them apart"
  );
}

#[test]
fn store_mode_stores_everything() {
  let (result, destination) = pack(
    "store_mode_stores_everything",
    &[("configs\\system.ltx", CONFIG)],
    |config| config.mode = ArchivePackMode::Store,
  );

  assert_eq!(result.files_compressed, 0);
  assert_eq!(result.files_stored, 1);
  assert_eq!(read(&open(&destination), "configs\\system.ltx"), CONFIG);
}

#[test]
fn reverts_to_stored_when_compression_does_not_pay() {
  let (result, destination) = pack(
    "reverts_to_stored_when_compression_does_not_pay",
    &[("configs\\tiny.ltx", b"[a]")],
    |_| {},
  );

  assert_eq!(result.files_compressed, 0, "three bytes cannot beat the margin");
  assert_eq!(read(&open(&destination), "configs\\tiny.ltx"), b"[a]");
}

#[test]
fn writes_an_empty_file_as_a_zero_length_entry() {
  let (_, destination) = pack(
    "writes_an_empty_file_as_a_zero_length_entry",
    &[("configs\\empty.ltx", b""), ("configs\\system.ltx", CONFIG)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);
  let empty: &ArchiveFileDescriptor = project.files.get("configs\\empty.ltx").expect("empty entry");

  assert_eq!(empty.size_real, 0);
  assert_eq!(empty.size_compressed, 0);
  assert!(!empty.is_directory, "a zero-byte file stays a file");
  assert!(project.files.get("configs\\").expect("directory entry").is_directory);
  // The neighbour must still be intact: an empty entry shares its offset with whatever follows.
  assert_eq!(read(&project, "configs\\system.ltx"), CONFIG);
}

#[test]
fn aliases_identical_payloads_to_one_copy() {
  let names: [&str; 3] = ["configs\\first.ltx", "configs\\second.ltx", "configs\\third.ltx"];
  let (result, destination) = pack(
    "aliases_identical_payloads_to_one_copy",
    &[(names[0], CONFIG), (names[1], CONFIG), (names[2], CONFIG)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_compressed, 1, "only the first copy costs payload bytes");
  assert_eq!(result.files_stored, 0);
  assert_eq!(result.files_aliased, 2);

  // The aliases inherit the compressed payload's stored size, not the size of the source they were read from.
  let shared: &ArchiveFileDescriptor = assert_one_payload(&project, &names);

  assert!(
    shared.size_compressed < shared.size_real,
    "the shared payload is the compressed one"
  );

  for name in names {
    assert_eq!(read(&project, name), CONFIG, "'{name}' still reads back");
  }
}

#[test]
fn aliases_a_stored_payload_too() {
  let names: [&str; 2] = ["textures\\wall.dds", "textures\\wall_copy.dds"];
  let (result, destination) = pack(
    "aliases_a_stored_payload_too",
    &[(names[0], BINARY), (names[1], BINARY)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_stored, 1);
  assert_eq!(result.files_aliased, 1);

  let shared: &ArchiveFileDescriptor = assert_one_payload(&project, &names);

  assert_eq!(
    shared.size_compressed, shared.size_real,
    "stored, so the alias declares equal sizes as well"
  );

  for name in names {
    assert_eq!(read(&project, name), BINARY, "'{name}' still reads back");
  }
}

#[test]
fn aliases_an_empty_file_onto_the_first_empty_one() {
  // xrCompress registers its empty-file branch for aliasing like any other payload, so the second empty entry is an
  // alias rather than a second zero-length payload, and the counts say so.
  let names: [&str; 2] = ["configs\\a_empty.ltx", "configs\\b_empty.ltx"];
  let (result, destination) = pack(
    "aliases_an_empty_file_onto_the_first_empty_one",
    &[(names[0], b""), (names[1], b""), ("configs\\system.ltx", CONFIG)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_stored, 1, "the first empty file");
  assert_eq!(result.files_compressed, 1, "the config");
  assert_eq!(result.files_aliased, 1, "the second empty file");

  let shared: &ArchiveFileDescriptor = assert_one_payload(&project, &names);

  assert_eq!(shared.size_real, 0);
  assert_eq!(shared.size_compressed, 0);

  for name in names {
    assert_eq!(read(&project, name), b"", "'{name}' reads back empty");
  }

  assert_eq!(
    read(&project, "configs\\system.ltx"),
    CONFIG,
    "the neighbour sharing the offset is intact"
  );
}

#[test]
fn does_not_alias_distinct_payloads_of_one_size() {
  const FIRST: &[u8] = &[0x11; 64];
  const SECOND: &[u8] = &[0x22; 64];

  // The table is keyed by size and checksum and a match is proven by the bytes, so equal size alone shares nothing.
  let (result, destination) = pack(
    "does_not_alias_distinct_payloads_of_one_size",
    &[("textures\\a.dds", FIRST), ("textures\\b.dds", SECOND)],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_aliased, 0);
  assert_eq!(result.files_stored, 2);
  assert_ne!(
    descriptor(&project, "textures\\a.dds").offset,
    descriptor(&project, "textures\\b.dds").offset,
    "each payload has its own bytes"
  );
  assert_eq!(read(&project, "textures\\a.dds"), FIRST);
  assert_eq!(read(&project, "textures\\b.dds"), SECOND);
}
