//! Reading an asset back from the mount that resolved it, and planning one source twice.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfError;

use crate::mount::XrayMountPlan;
use crate::vfs::tests::fixtures::directory;
use crate::{XrayAsset, XrayLogicalPath, XrayLookupScope, XrayMountId, XrayVfs};

#[test]
fn reads_an_asset_from_the_source_that_resolved_it() {
  let root: PathBuf = directory("read_asset", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("root mounts");

  let asset: XrayAsset = vfs
    .find_in(&XrayLookupScope::all(), "configs\\system.ltx")
    .expect("lookup")
    .expect("resolves");

  // The helper writes the tree's name as each file's contents, so this proves which source answered.
  assert_eq!(vfs.read_asset_bytes(&asset).expect("reads"), b"read_asset");
}

#[test]
fn reading_an_asset_from_another_vfs_is_not_found_rather_than_wrong_bytes() {
  // The asset names a container this VFS does not hold, so answering with whatever wins here would be a silent
  // substitution of one mount's bytes for another's.
  let other: PathBuf = directory("read_asset_foreign", &["configs/system.ltx"]);
  let mut elsewhere: XrayVfs = XrayVfs::new();

  elsewhere.mount_directory("", &other).expect("root mounts");

  let asset: XrayAsset = elsewhere
    .find_in(&XrayLookupScope::all(), "configs\\system.ltx")
    .expect("lookup")
    .expect("resolves");

  assert!(XrayVfs::new().read_asset_bytes(&asset).is_err());
}

#[test]
fn reads_an_asset_from_the_mount_of_its_root_that_holds_it() {
  // One directory at two bases is two mounts sharing a container root. The asset found through the second must be read
  // through it: the first would look `configs\system.ltx` up below the root and read the other file.
  let root: PathBuf = directory("read_asset_rebased", &["configs/system.ltx", "system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("root mounts");
  vfs.mount_directory("other", &root).expect("root mounts at a base");

  fs::write(root.join("system.ltx"), b"based").expect("the based copy is written");

  let asset: XrayAsset = vfs
    .find_in(&XrayLookupScope::all(), "other\\system.ltx")
    .expect("lookup")
    .expect("resolves");

  assert_eq!(vfs.read_asset_bytes(&asset).expect("reads"), b"based");

  let missing: XrayAsset = XrayAsset::new(
    XrayLogicalPath::new("other\\absent.ltx").expect("valid path"),
    asset.get_container().clone(),
  );

  assert!(
    vfs
      .read_asset_bytes(&missing)
      .is_err_and(|error| matches!(error, XrfError::NotFound { .. })),
    "a path no mount of the root holds is absent"
  );
}

#[test]
fn planning_the_same_source_twice_reuses_its_mount() {
  // A viewer keeps one VFS across requests and plans the model's root each time. Without this, every request re-walks
  // the tree and appends another mount, so memory and mount count grow for as long as the session lasts.
  let root: PathBuf = directory("plan_reuse", &["textures/wpn/wpn_ak74.dds"]);
  let plan: XrayMountPlan = XrayMountPlan::root(&root).expect("plan");
  let mut vfs: XrayVfs = XrayVfs::new();

  let first: Vec<XrayMountId> = vfs.mount_plan(&plan).expect("first mount");
  let second: Vec<XrayMountId> = vfs.mount_plan(&plan).expect("second mount");

  assert_eq!(first, second, "the same mount answers both plans");
  assert_eq!(vfs.get_mounts().len(), 1, "planning twice does not append a duplicate");
}
