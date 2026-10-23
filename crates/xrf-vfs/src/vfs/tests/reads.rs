//! Reading an asset back from the source that resolved it, and planning one source twice.

use std::path::PathBuf;

use crate::mount::XrayMountPlan;
use crate::vfs::tests::fake_source::directory;
use crate::{XrayAsset, XrayLookupScope, XrayMountId, XrayVfs};

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
