//! The trees, search and draw every surface test shares.

use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_vfs::{XrayMountId, XraySearch, XrayVfs};

use crate::fixtures::FixtureTree;
use crate::tests::material_search::search_over;
use crate::{XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceResolver, XrayTextureScope};

/// What every deferred cut-out tests against, whatever the blender authored.
pub(crate) const CUT_OUT: XraySurfaceDraw = XraySurfaceDraw::AlphaTested {
  reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
};

/// A tree whose library defines exactly `blenders`.
pub(crate) fn library(case: &str, blenders: &[ShaderBlenderFixture]) -> FixtureTree {
  FixtureTree::new(case).with_shader_library(blenders)
}

/// Opens `tree` and describes one shader name in it, dressed with nothing.
pub(crate) fn describe(tree: &FixtureTree, shader: &str) -> XraySurfaceDescriptor {
  describe_dressed(tree, shader, &[])
}

/// Opens `tree` and describes one shader name in it, dressed with textures.
pub(crate) fn describe_dressed(tree: &FixtureTree, shader: &str, textures: &[&str]) -> XraySurfaceDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let search: XraySearch = search_over(vfs, id);
  let textures: Vec<String> = textures.iter().map(|it| String::from(*it)).collect();

  XraySurfaceResolver::open(&search, XrayTextureScope::shared()).describe(shader, &textures)
}
