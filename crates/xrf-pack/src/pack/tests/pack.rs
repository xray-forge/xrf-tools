//! Round-trips the writer through the reader, which is the only check that matters: an archive is
//! correct when the code that mounts one can read it back.
//!
//! Source trees and volumes are built under the per-process scratch tree, scoped per test.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayProbeStep, XrayRoots, XrayVfs};

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory, ArchivePackMode, VOLUME_SIZE_MAX};
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

/// Assert every produced volume file is within the cap its configuration advertised.
///
/// File lengths, not the writer's own accounting: the cap covers the descriptor chunk appended at close, which is
/// what a check against reported sizes or entry counts would miss.
fn assert_volumes_within(result: &ArchivePackResult, max_volume_size: u64) {
  for volume in &result.volumes {
    let length: u64 = fs::metadata(volume).expect("volume metadata").len();

    assert!(
      length <= max_volume_size,
      "'{}' is {length} bytes, past the configured maximum of {max_volume_size}",
      format_path(volume)
    );
  }
}

/// Distinct payloads of one size, since identical ones would alias onto a single copy and never split.
fn distinct_files(count: u8, extension: &str, size: usize) -> Vec<(String, Vec<u8>)> {
  (0..count)
    .map(|index| (format!("textures\\tile_{index}.{extension}"), vec![b'a' + index; size]))
    .collect()
}

fn borrow_files(files: &[(String, Vec<u8>)]) -> Vec<(&str, &[u8])> {
  files
    .iter()
    .map(|(name, contents)| (name.as_str(), contents.as_slice()))
    .collect()
}

fn assert_pack_rejects_invalid_volume_size(scope: &str, max_volume_size: u64) {
  let source: PathBuf = create_source(scope, &[("configs\\system.ltx", CONFIG)]);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));
  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  let _ = fs::remove_dir_all(&destination);

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];
  config.max_volume_size = max_volume_size;

  assert!(matches!(ArchivePacker::pack(&config), Err(XrfError::Invalid { .. })));
  assert!(!destination.exists(), "an invalid configuration creates no destination");
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
fn rejects_a_zero_volume_size_before_writing() {
  assert_pack_rejects_invalid_volume_size("rejects_a_zero_volume_size_before_writing", 0);
}

#[test]
fn rejects_an_oversized_volume_before_writing() {
  assert_pack_rejects_invalid_volume_size("rejects_an_oversized_volume_before_writing", VOLUME_SIZE_MAX + 1);
}

#[test]
fn refuses_a_volume_name_that_would_leave_the_destination_before_writing() {
  let scope: &str = "refuses_a_volume_name_that_would_leave_the_destination_before_writing";
  let source: PathBuf = create_source(scope, &[("configs\\system.ltx", CONFIG)]);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  for name in ["", "..\\outside", "nested\\name", "\\rooted", "C:\\outside"] {
    let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, name);

    config.include_directories = vec![ArchivePackDirectory {
      path: String::new(),
      is_recursive: true,
    }];

    assert!(
      matches!(ArchivePacker::pack(&config), Err(XrfError::Invalid { .. })),
      "'{name}' must not name a volume"
    );
  }

  assert!(!destination.exists(), "a refused name creates no destination");
  // The destination's own parent exists, holding the source tree, so a name that walked out of it would land here.
  assert!(
    !destination
      .parent()
      .expect("destination parent")
      .join("outside.db0")
      .exists(),
    "and writes nothing beside it"
  );
}

#[test]
fn every_published_volume_is_a_direct_child_of_the_destination() {
  let scope: &str = "every_published_volume_is_a_direct_child_of_the_destination";

  let files: Vec<(String, Vec<u8>)> = distinct_files(8, "dds", 4096);
  let borrowed: Vec<(&str, &[u8])> = borrow_files(&files);

  // Both publication paths: the file each volume is created as, and the rename a lone volume ends under.
  let split: (ArchivePackResult, PathBuf) = pack(&format!("{scope}/split"), &borrowed, |config| {
    config.max_volume_size = 8 * 1024;
  });
  let single: (ArchivePackResult, PathBuf) =
    pack(&format!("{scope}/single"), &[("configs\\system.ltx", CONFIG)], |_| {});

  assert!(split.0.volumes.len() > 1, "the split set spans several volumes");
  assert_eq!(
    single.0.volumes,
    vec![single.1.join("packed.db")],
    "the lone volume is renamed"
  );

  for (result, destination) in [split, single] {
    for volume in &result.volumes {
      assert_eq!(
        volume.parent(),
        Some(destination.as_path()),
        "a volume stays in the destination"
      );
      assert!(volume.is_file(), "and is the file that was written");
    }
  }
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
fn refuses_an_entry_no_volume_of_that_size_could_hold() {
  let scope: &str = "refuses_an_entry_no_volume_of_that_size_could_hold";
  let source: PathBuf = create_source(scope, &[("textures\\wall.dds", &[0x44; 4096])]);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));
  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  let _ = fs::remove_dir_all(&destination);

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];
  // Room for the chunks every volume carries, and none for this entry. Isolating it would publish a volume
  // eight times the advertised cap, so the pack is refused instead.
  config.max_volume_size = 512;

  assert!(matches!(ArchivePacker::pack(&config), Err(XrfError::Invalid { .. })));
}

#[test]
fn refuses_a_cap_no_volume_could_open_within() {
  // Smaller than the header chunk and the two chunk headers every volume repeats, so no split can help.
  assert_pack_rejects_invalid_volume_size("refuses_a_cap_no_volume_could_open_within", 64);
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

#[test]
fn an_excluded_directory_does_not_reach_the_neighbour_it_shares_a_prefix_with() {
  let (result, destination) = pack(
    "an_excluded_directory_does_not_reach_the_neighbour_it_shares_a_prefix_with",
    &[
      ("configs\\system.ltx", CONFIG),
      ("configs\\weapons\\w_ak74.ltx", CONFIG),
      ("configs_backup\\system.ltx", CONFIG),
    ],
    |config| {
      // Spelled unlike the tree on disk, because the engine resolves a name either way.
      config.exclude_directories = vec![ArchivePackDirectory {
        path: String::from("Configs"),
        is_recursive: true,
      }];
    },
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_total, 1, "only the backup tree survives the rule");
  assert_eq!(read(&project, "configs_backup\\system.ltx"), CONFIG);

  for name in ["configs\\", "configs\\system.ltx", "configs\\weapons\\w_ak74.ltx"] {
    assert!(!project.files.contains_key(name), "'{name}' is excluded");
  }
}
