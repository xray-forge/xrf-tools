//! Round-trips the writer through the reader, which is the only check that matters: an archive is correct when the
//! code that mounts one can read it back.
//!
//! Also where a volume is reached from: a project opened at one file, at a directory of them, or at a root a probe
//! plan has to recognise. What each entry costs is `payload`, what the rules selected is `selection`.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayAsset, XrayMountMode, XrayProbe, XrayProbeStep, XrayRoots, XrayVfs};

use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::tests::fixtures::{BINARY, CONFIG, create_source, open, pack, read};
use crate::pack::{ArchivePackResult, ArchivePacker};

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
