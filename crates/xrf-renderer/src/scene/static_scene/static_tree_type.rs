use glam::Vec4;
use xrf_renderer_core::ProxyHandle;

use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;

/// One part of a tree type as its bake draws it: its whole-detail slot's clusters, under its batch's draw list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticTreePart {
  /// The batch's draw list, `StaticBatch::get_index`.
  pub batch: u32,
  pub first_cluster: u32,
  pub cluster_count: u32,
}

/// A tree type as the scene holds it for its bake: one tree's parts, from the sector that first stood one, and the
/// sphere they fill in the type's own space.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticTreeType {
  pub parts: Vec<StaticTreePart>,
  pub sphere: Vec4,
  /// The sector whose clusters the parts are, which takes them away when it leaves.
  pub sector: ProxyHandle<StaticSectorProxy>,
}
