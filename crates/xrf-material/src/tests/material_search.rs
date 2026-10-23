//! The tree, search and names every test in this module shares.

use std::sync::Arc;

use xrf_thm::ThmBumpMode;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayResolution, XraySearch, XrayVfs};

use crate::fixtures::{FixtureTree, ThmFixture};
use crate::{XrayMaterialDescriptor, XrayMaterialResolver, XrayTextureScope};

pub(crate) const BASE: &str = "act\\act_stalker";
pub(crate) const BUMP: &str = "act\\act_stalker_bump";
pub(crate) const COMPANION: &str = "act\\act_stalker_bump#";

/// One mounted tree and a search naming it.
pub(crate) fn search_over(vfs: impl Into<Arc<XrayVfs>>, id: XrayMountId) -> XraySearch {
  XraySearch::new(vfs).with_step("tree", XrayLookupScope::only([id]))
}

pub(crate) fn describe(tree: &FixtureTree) -> XrayMaterialDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  XrayMaterialResolver::describe_texture(&search_over(vfs, id), &XrayTextureScope::shared(), BASE)
}

pub(crate) fn used_bump() -> ThmFixture {
  ThmFixture::image().with_bump(ThmBumpMode::Use, BUMP)
}

pub(crate) fn located_path(resolution: &XrayResolution) -> Option<&str> {
  resolution.get_asset().map(|asset| asset.get_logical_path().as_str())
}
