//! The tree, probe and names every test in this module shares.

use xrf_db::ThmBumpChunk;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayProbe, XrayResolution, XrayVfs};

use crate::fixtures::{ThmFixture, ThmFixtureTree};
use crate::{XrayMaterialDescriptor, XrayMaterialResolver};

pub(crate) const BASE: &str = "act\\act_stalker";
pub(crate) const BUMP: &str = "act\\act_stalker_bump";
pub(crate) const COMPANION: &str = "act\\act_stalker_bump#";

/// One mounted tree and a probe naming it.
pub(crate) fn probe_over(vfs: &XrayVfs, id: XrayMountId) -> XrayProbe<'_> {
  vfs.probe().with_step("tree", XrayLookupScope::only([id]))
}

pub(crate) fn describe(tree: &ThmFixtureTree) -> XrayMaterialDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  XrayMaterialResolver::describe_texture(&probe_over(&vfs, id), BASE)
}

pub(crate) fn used_bump() -> ThmFixture {
  ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP)
}

pub(crate) fn located_path(resolution: &XrayResolution) -> Option<&str> {
  resolution.get_asset().map(|asset| asset.get_logical_path().as_str())
}
