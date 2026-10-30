use std::collections::BTreeMap;

use xrf_renderer_core::ProxyHandle;

use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_tree_type::StaticTreeType;

/// The level's tree types as its standing sectors give them, each as the first sector standing one of its trees gives
/// it: what a type bakes from, again from the next sector standing one where its own has left.
#[derive(Debug, Default)]
pub struct StaticTreeTypes {
  types: BTreeMap<u32, StaticTreeType>,
}

impl StaticTreeTypes {
  /// Whether a standing sector gave a type already, so a later sector's tree of it gives nothing.
  pub fn has(&self, kind: u32) -> bool {
    self.types.contains_key(&kind)
  }

  /// Keeps a type's parts, where no standing sector gave them yet.
  pub fn note(&mut self, kind: u32, tree: StaticTreeType) {
    self.types.entry(kind).or_insert(tree);
  }

  /// Forgets the types a leaving sector gave, whose clusters go with it.
  pub fn forget(&mut self, sector: ProxyHandle<StaticSectorProxy>) {
    self.types.retain(|_, tree| tree.sector != sector);
  }

  /// The types whose parts a standing sector holds, by type.
  pub fn list(&self) -> impl Iterator<Item = (u32, &StaticTreeType)> {
    self.types.iter().map(|(kind, tree)| (*kind, tree))
  }

  /// How many types the standing sectors give.
  pub fn len(&self) -> usize {
    self.types.len()
  }

  pub fn is_empty(&self) -> bool {
    self.types.is_empty()
  }
}
