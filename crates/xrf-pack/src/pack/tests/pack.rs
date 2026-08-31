//! Round-trips the writer through the reader, which is the only check that matters: an archive is
//! correct when the code that mounts one can read it back.
//!
//! What a volume may cost is `volume_size`; this is what it holds.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayAsset, XrayMountMode, XrayProbe, XrayProbeStep, XrayRoots, XrayVfs};

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory, ArchivePackMode};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_packer::ArchivePacker;
use crate::pack::tests::fixtures::{
  BINARY, CONFIG, allow_directory, borrow_files, create_config, create_source, deny_directory, distinct_files, open,
  pack, read,
};

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
    let steps: Vec<XrayProbeStep> = XrayRoots::one(root.to_path_buf(), XrayMountMode::Auto)
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
fn a_volume_in_a_subdirectory_is_read_through_the_project_root() {
  // The explorer discovers volumes recursively and then mounts the project's root to preview an entry. Planned as one
  // shallow volume set, that root sees only the volumes directly under it, and every entry stored deeper is listed,
  // sized and extractable but previews as missing.
  let scope: &str = "a_volume_in_a_subdirectory_is_read_through_the_project_root";
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  pack_volume(scope, "shaders", &[("shaders\\r1\\clouds.vs", CONFIG)]);

  // Moved below the root the way an installation stores its own: `db\textures\textures.db0` beside `db\shaders.db0`.
  let packed: PathBuf = pack_volume(scope, "textures", &[("textures\\act.dds", BINARY)]);
  let nested: PathBuf = destination.join("textures");

  fs::create_dir_all(&nested).expect("volume subdirectory");
  fs::rename(&packed, nested.join(packed.file_name().expect("volume name"))).expect("volume moves");

  let project: ArchiveProject = open(&destination);

  assert_eq!(project.archives.len(), 2, "both volumes belong to the project");
  assert_eq!(project.root, destination);

  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XrayProbeStep> = XrayRoots::one(project.root.clone(), XrayMountMode::Volumes)
    .to_probe_plan()
    .expect("roots plan")
    .mount_into(&mut vfs)
    .expect("plan mounts");
  let probe: XrayProbe = vfs.probe().with_steps(steps);

  for name in ["shaders\\r1\\clouds.vs", "textures\\act.dds"] {
    let asset: XrayAsset = probe
      .find(name)
      .expect("lookup succeeds")
      .get_asset()
      .cloned()
      .unwrap_or_else(|| panic!("'{name}' is listed by the project, so it must resolve through its root"));

    assert_eq!(
      probe.read_asset_bytes(&asset).expect("asset reads"),
      read(&project, name),
      "'{name}' reads the same bytes through the root and through the project"
    );
  }
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
  let (config, _) = create_config("refuses_a_name_it_cannot_encode", &[("configs\\ロゴ.ltx", CONFIG)]);

  // Silently mangling a name would produce an archive the engine cannot resolve by that name.
  assert!(ArchivePacker::pack(&config).is_err());
}

/// A packed volume set is read as a complete build of what the configuration selected, so a source subtree the walk
/// cannot enumerate has to end the run. Filtered out, packing reported success over an archive silently missing
/// everything below the unreadable directory.
#[test]
fn refuses_to_pack_a_source_subtree_it_cannot_read() {
  let (config, destination) = create_config(
    "refuses_to_pack_a_source_subtree_it_cannot_read",
    &[
      ("configs\\system.ltx", CONFIG),
      ("scripts\\locked\\hidden.script", CONFIG),
    ],
  );
  let locked: PathBuf = config.source.join("scripts").join("locked");

  if !deny_directory(&locked) {
    return;
  }

  let result: Result<ArchivePackResult, XrfError> = ArchivePacker::pack(&config);

  // Restored before asserting, so a failed expectation still leaves a tree the next run can remove.
  allow_directory(&locked);

  assert!(
    result.is_err(),
    "an unreadable subtree is a packing failure, not an omission"
  );
  assert!(
    !destination.exists(),
    "and selection fails before anything is published"
  );
}

/// The other half of that rule: a subtree a recursive exclusion drops holds nothing that could be selected, so it is
/// never read at all and does not have to be readable for the rest of the tree to pack.
#[test]
fn does_not_read_a_recursively_excluded_subtree() {
  let (mut config, destination) = create_config(
    "does_not_read_a_recursively_excluded_subtree",
    &[
      ("configs\\system.ltx", CONFIG),
      ("scripts\\locked\\hidden.script", CONFIG),
    ],
  );
  let locked: PathBuf = config.source.join("scripts").join("locked");

  config.exclude_directories = vec![ArchivePackDirectory {
    path: String::from("scripts"),
    is_recursive: true,
  }];

  if !deny_directory(&locked) {
    return;
  }

  let result: Result<ArchivePackResult, XrfError> = ArchivePacker::pack(&config);

  allow_directory(&locked);

  assert_eq!(
    result.expect("the excluded subtree is never read").files_total,
    1,
    "only the configuration outside the rule is packed"
  );
  assert!(open(&destination).files.contains_key("configs\\system.ltx"));
}

/// A Unix filename is bytes, not text, so a source file can be perfectly valid and have no archive name at all. The
/// writer already refuses a name it cannot encode as windows-1251; this one never reached it, because a host path
/// that is not valid Unicode produced no name to refuse.
#[test]
#[cfg(unix)]
fn refuses_a_source_file_whose_host_name_is_not_valid_unicode() {
  use std::ffi::OsStr;
  use std::os::unix::ffi::OsStrExt;

  let (config, destination) = create_config(
    "refuses_a_source_file_whose_host_name_is_not_valid_unicode",
    &[("configs\\system.ltx", CONFIG)],
  );

  fs::write(
    config.source.join("configs").join(OsStr::from_bytes(b"broken\xff.ltx")),
    CONFIG,
  )
  .expect("source file");

  assert!(
    ArchivePacker::pack(&config).is_err(),
    "a file with no archive name is reported rather than dropped"
  );
  assert!(!destination.exists(), "and nothing is published");
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
