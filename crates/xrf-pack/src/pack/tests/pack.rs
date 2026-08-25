//! Round-trips the writer through the reader, which is the only check that matters: an archive is
//! correct when the code that mounts one can read it back.
//!
//! Source trees and volumes are built under the per-process scratch tree, scoped per test.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayProbeStep, XrayRoots, XrayVfs};

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory, ArchivePackMode};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_packer::ArchivePacker;

/// A configuration fragment large enough that compressing it actually pays off.
const CONFIG: &[u8] = b"[section]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

/// Bytes with no structure to exploit, standing in for a texture or a mesh.
const BINARY: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x01, 0x02, 0x03, 0xfe, 0xff, 0x00];

/// Build a source tree under this test's scratch directory and return its root.
fn create_source(scope: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/gamedata"));

  let _ = fs::remove_dir_all(&root);

  for (name, contents) in files {
    let path: PathBuf = root.join(name.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("source directory");
    fs::write(&path, contents).expect("source file");
  }

  root
}

/// Pack a freshly built source tree, returning what was written and where.
fn pack(
  scope: &str,
  files: &[(&str, &[u8])],
  configure: impl FnOnce(&mut ArchivePackConfig),
) -> (ArchivePackResult, PathBuf) {
  let source: PathBuf = create_source(scope, files);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  // Every test packs the whole tree unless it says otherwise.
  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];

  configure(&mut config);

  let result: ArchivePackResult = ArchivePacker::pack(&config).expect("source tree packs");

  (result, destination)
}

fn open(destination: &Path) -> ArchiveProject {
  ArchiveProject::new(destination).expect("written archive opens")
}

fn read(project: &ArchiveProject, name: &str) -> Vec<u8> {
  project
    .read_file_bytes(name)
    .unwrap_or_else(|error| panic!("archive holds '{name}': {error}"))
}

/// Pack one source tree into a destination shared with other volumes, under its own volume name.
fn pack_volume(scope: &str, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let source: PathBuf = create_source(&format!("{scope}/{name}"), files);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, name);

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];

  let result: ArchivePackResult = ArchivePacker::pack(&config).expect("source tree packs");

  result.volumes.first().expect("one volume written").clone()
}

#[test]
fn opening_one_volume_opens_only_that_volume() {
  // What the explorer's archive mode asks for: the volume the user named, not the directory it happens to sit in.
  let scope: &str = "opening_one_volume_opens_only_that_volume";
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  let first: PathBuf = pack_volume(scope, "first", &[("configs\\first.ltx", CONFIG)]);
  let second: PathBuf = pack_volume(scope, "second", &[("configs\\second.ltx", CONFIG)]);

  assert_ne!(first, second, "the two volumes share a directory");
  assert_eq!(open(&destination).archives.len(), 2, "the directory holds both");

  let project: ArchiveProject = open(&first);

  assert_eq!(project.archives.len(), 1);
  assert_eq!(project.archives[0].path, first);

  // The root is what a caller mounts to read an entry back out. Left as the parent directory it would reach the
  // sibling volume too, and an entry both hold would answer out of whichever one sorted last.
  assert_eq!(project.root, first);

  assert_eq!(read(&project, "configs\\first.ltx"), CONFIG);
  assert!(
    !project.files.contains_key("configs\\second.ltx"),
    "the sibling volume's entries stay out of a project opened from one file"
  );
}

#[test]
fn a_named_volume_is_searched_as_a_root() {
  // The explorer mounts a project's root to read an entry's bytes back, and for a single-volume project that root is
  // the volume. A root planner that only recognised directories would search nothing and report every entry missing.
  let scope: &str = "a_named_volume_is_searched_as_a_root";
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  let first: PathBuf = pack_volume(scope, "first", &[("configs\\first.ltx", CONFIG)]);
  let sibling: PathBuf = pack_volume(scope, "second", &[("configs\\second.ltx", CONFIG)]);

  // Searched the way the asset commands search: a probe plan over the roots, not a plain mount.
  fn find(root: &Path, logical_path: &str) -> bool {
    let mut vfs: XrayVfs = XrayVfs::new();
    let steps: Vec<XrayProbeStep> = XrayRoots::one(root.display().to_string(), XrayMountMode::Auto)
      .to_probe_plan()
      .expect("roots plan")
      .mount_into(&mut vfs)
      .expect("plan mounts");

    vfs
      .probe()
      .with_steps(steps)
      .find(logical_path)
      .expect("lookup succeeds")
      .get_asset()
      .is_some()
  }

  assert!(find(&first, "configs\\first.ltx"), "its own entry is found");
  assert!(
    !find(&first, "configs\\second.ltx"),
    "the volume beside it is not searched"
  );

  // The directory holding both is a wider root on purpose, and is what a directory-opened project still mounts.
  assert!(find(sibling.parent().expect("volume parent"), "configs\\second.ltx"));
}

#[test]
fn packs_a_tree_the_reader_reads_back() {
  let files: [(&str, &[u8]); 3] = [
    ("configs\\system.ltx", CONFIG),
    ("configs\\weapons\\w_ak74.ltx", CONFIG),
    ("textures\\wall.dds", BINARY),
  ];
  let (result, destination) = pack("packs_a_tree_the_reader_reads_back", &files, |_| {});
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_total, 3);
  assert_eq!(result.volumes.len(), 1);

  for (name, contents) in files {
    assert_eq!(read(&project, name), contents, "'{name}' reads back unchanged");
  }
}

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
  let (result, destination) = pack(
    "aliases_identical_payloads_to_one_copy",
    &[
      ("configs\\first.ltx", CONFIG),
      ("configs\\second.ltx", CONFIG),
      ("configs\\third.ltx", CONFIG),
    ],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_aliased, 2, "only the first copy costs payload bytes");

  for name in ["configs\\first.ltx", "configs\\second.ltx", "configs\\third.ltx"] {
    assert_eq!(read(&project, name), CONFIG, "'{name}' still reads back");
  }
}

#[test]
fn keeps_names_the_engine_can_read() {
  let name: &str = "configs\\текст\\диалог.ltx";
  let (_, destination) = pack("keeps_names_the_engine_can_read", &[(name, CONFIG)], |_| {});
  let project: ArchiveProject = open(&destination);

  // The reader decodes names as windows-1251, so a Cyrillic name only survives if it was written so.
  assert_eq!(read(&project, name), CONFIG);
}

#[test]
fn refuses_a_name_it_cannot_encode() {
  let source: PathBuf = create_source("refuses_a_name_it_cannot_encode", &[("configs\\ロゴ.ltx", CONFIG)]);
  let destination: PathBuf = build_absolute_generated_test_resource_path("refuses_a_name_it_cannot_encode/db");
  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];

  // Silently mangling a name would produce an archive the engine cannot resolve by that name.
  assert!(ArchivePacker::pack(&config).is_err());
}

#[test]
fn a_lone_volume_carries_no_index() {
  let (result, destination) = pack(
    "a_lone_volume_carries_no_index",
    &[("configs\\system.ltx", CONFIG)],
    |_| {},
  );

  // What the shipped games do: `configs.db`, not `configs.db0`, when there is only one.
  assert_eq!(result.volumes[0].file_name().expect("name"), "packed.db");
  assert!(destination.join("packed.db").is_file());
  assert!(!destination.join("packed.db0").exists(), "the indexed name is gone");
  assert_eq!(read(&open(&destination), "configs\\system.ltx"), CONFIG);
}

#[test]
fn splits_volumes_at_the_configured_size() {
  // Distinct payloads, or the writer would rightly alias them all onto one copy and never split.
  let files: Vec<(String, Vec<u8>)> = (0..8u8)
    .map(|index| (format!("textures\\tile_{index}.dds"), vec![b'a' + index; 4096]))
    .collect();
  let borrowed: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, contents)| (name.as_str(), contents.as_slice()))
    .collect();

  let (result, destination) = pack("splits_volumes_at_the_configured_size", &borrowed, |config| {
    config.max_volume_size = 8 * 1024;
  });
  let project: ArchiveProject = open(&destination);

  assert!(result.volumes.len() > 1, "the set spans several volumes");
  assert_eq!(project.archives.len(), result.volumes.len());

  // A real set keeps its indices, starting at zero.
  assert_eq!(result.volumes[0].file_name().expect("name"), "packed.db0");
  assert_eq!(result.volumes[1].file_name().expect("name"), "packed.db1");

  for (name, contents) in &files {
    assert_eq!(read(&project, name), *contents, "'{name}' survives the split");
  }
}

#[test]
fn carries_the_header_the_engine_mounts_by() {
  let (_, destination) = pack(
    "carries_the_header_the_engine_mounts_by",
    &[("configs\\system.ltx", CONFIG)],
    |config| {
      config.header = Some(String::from(
        "[header]\r\nauto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n",
      ));
    },
  );
  let project: ArchiveProject = open(&destination);

  // The reader takes the mount root out of that header, so reading it back proves the chunk landed.
  assert_eq!(project.archives[0].output_root_path, Path::new("gamedata\\"));
}

#[test]
fn leaves_out_what_the_engine_rebuilds() {
  let (result, destination) = pack(
    "leaves_out_what_the_engine_rebuilds",
    &[
      ("configs\\system.ltx", CONFIG),
      ("readme.txt", b"notes"),
      ("textures\\lod\\lod_wall.dds", BINARY),
    ],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_total, 1, "only the config is packed");
  assert_eq!(result.files_skipped, 2);
  assert!(project.files.contains_key("configs\\system.ltx"));
  assert!(!project.files.contains_key("readme.txt"));
}
