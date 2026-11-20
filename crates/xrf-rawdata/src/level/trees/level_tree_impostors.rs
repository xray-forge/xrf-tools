use std::collections::HashMap;

use xrf_ogf::OgfLodDefinitionChunk;

use crate::level::trees::level_tree_impostor_atlas::LevelTreeImpostorAtlas;

/// What a level holds of its trees' impostors: the atlas, and each `MT_LOD`'s definition by its visual's index.
pub(crate) struct LevelTreeImpostors<'a> {
  pub(crate) atlas: &'a LevelTreeImpostorAtlas,
  pub(crate) lods: HashMap<u32, &'a OgfLodDefinitionChunk>,
}
