//! Volumes of one set mounted under different entry points: the engine registers each entry under its own volume's
//! entry point, so two volumes holding the same authored name there hold two files, not a base and its patch.

use std::fs;
use std::path::PathBuf;

use xrf_archive::ArchiveProject;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayAsset, XrayMountMode, XrayRoots, XraySearch, XraySearchStep, XrayVfs};

use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::tests::fixtures::{create_source, open, read};
use crate::pack::{ArchivePackResult, ArchivePacker};
use crate::unpack::{ArchiveUnpackResult, ArchiveUnpacker};

const CONFIGS: &[u8] = b"[configs]\n";
const SCRIPTS: &[u8] = b"[scripts]\n";

/// A set directory of its own, emptied first so a re-run reads only what this run packs.
fn destination(scope: &str) -> PathBuf {
  let destination: PathBuf = GeneratedResources::path(&format!("entry_points/{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  destination
}

/// Pack one file into a volume of its own under `entry_point`, beside the set's other volumes.
fn pack_volume(scope: &str, name: &str, entry_point: &str, file: (&str, &[u8])) -> PathBuf {
  let source: PathBuf = create_source(&format!("entry_points/{scope}/{name}"), &[file]);
  let mut config: ArchivePackConfig = ArchivePackConfig::new(
    &source,
    GeneratedResources::path(&format!("entry_points/{scope}/db")),
    name,
  );

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];
  config.header = Some(format!(
    "[header]\r\nauto_load = true\r\nentry_point = {entry_point}\r\n"
  ));

  let result: ArchivePackResult = ArchivePacker::pack(&config).expect("source tree packs");

  result.volumes.first().expect("one volume written").clone()
}

/// The bytes a search over the set's volumes answers an engine path with, as the explorer and the game tools read it.
fn find(root: PathBuf, logical_path: &str) -> Option<Vec<u8>> {
  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XraySearchStep> = XrayRoots::one(root, XrayMountMode::Volumes)
    .to_search_plan()
    .expect("roots plan")
    .mount_into(&mut vfs)
    .expect("plan mounts");
  let search: XraySearch = XraySearch::new(vfs).with_steps(steps);
  let asset: XrayAsset = search.find(logical_path).expect("lookup succeeds")?;

  Some(search.read_asset(&asset).expect("asset reads"))
}

#[test]
fn distinct_entry_points_keep_both_files() {
  let scope: &str = "distinct";
  let destination: PathBuf = destination(scope);

  pack_volume(scope, "a", "$fs_root$\\gamedata\\configs\\", ("common.ltx", CONFIGS));
  pack_volume(scope, "b", "$fs_root$\\gamedata\\scripts\\", ("common.ltx", SCRIPTS));

  let project: ArchiveProject = open(&destination);

  assert_eq!(project.archives.len(), 2);
  assert_eq!(project.count_files(), 2, "one file per entry point");
  assert!(project.shadowed.is_empty(), "neither file is a patch of the other");
  assert_eq!(read(&project, "configs\\common.ltx"), CONFIGS);
  assert_eq!(read(&project, "scripts\\common.ltx"), SCRIPTS);
  assert_eq!(
    &*project.files["configs\\common.ltx"].authored_name, "common.ltx",
    "the row name stays as the volume records it"
  );

  let unpacked: PathBuf = GeneratedResources::path(&format!("entry_points/{scope}/unpacked"));
  let _ = fs::remove_dir_all(&unpacked);
  let result: ArchiveUnpackResult = ArchiveUnpacker::unpack(&project, &unpacked).expect("set unpacks");

  assert_eq!(result.files_unpacked, 2);
  assert_eq!(
    fs::read(unpacked.join("gamedata/configs/common.ltx")).expect("configs file"),
    CONFIGS
  );
  assert_eq!(
    fs::read(unpacked.join("gamedata/scripts/common.ltx")).expect("scripts file"),
    SCRIPTS
  );

  assert_eq!(
    find(destination.clone(), "configs\\common.ltx").as_deref(),
    Some(CONFIGS)
  );
  assert_eq!(find(destination, "scripts\\common.ltx").as_deref(), Some(SCRIPTS));
}

#[test]
fn the_same_entry_point_still_uses_the_later_volume() {
  let scope: &str = "same";
  let destination: PathBuf = destination(scope);

  // Spelled differently on purpose: the engine compares its paths without case, and `gamedata` is `$fs_root$\gamedata\`.
  pack_volume(scope, "a", "$fs_root$\\gamedata\\", ("configs\\common.ltx", CONFIGS));
  pack_volume(scope, "b", "gamedata", ("configs\\common.ltx", SCRIPTS));

  let project: ArchiveProject = open(&destination);

  assert_eq!(project.count_files(), 1, "one mounted file");
  assert_eq!(read(&project, "configs\\common.ltx"), SCRIPTS, "the later volume wins");
  assert_eq!(
    project.shadowed.len(),
    1,
    "the earlier copy is the one the patch buried"
  );
  assert_eq!(project.shadowed[0].volume, 0);
  assert_eq!(find(destination, "configs\\common.ltx").as_deref(), Some(SCRIPTS));
}

#[test]
fn a_volume_mounted_below_gamedata_is_found_at_its_engine_path() {
  let scope: &str = "below";
  let destination: PathBuf = destination(scope);

  pack_volume(scope, "a", "$fs_root$\\gamedata\\configs\\", ("common.ltx", CONFIGS));

  let project: ArchiveProject = open(&destination);

  // Alone, the volume's whole entry point is shared by the set, so the project names its entry as authored.
  assert_eq!(read(&project, "common.ltx"), CONFIGS);
  assert_eq!(
    find(destination.clone(), "configs\\common.ltx").as_deref(),
    Some(CONFIGS)
  );
  assert_eq!(
    find(destination, "common.ltx"),
    None,
    "nothing is mounted at the authored name alone"
  );
}
