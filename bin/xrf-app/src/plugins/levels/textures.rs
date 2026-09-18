//! What a level's surfaces are dressed with, resolved once for the whole shader table.

use std::collections::BTreeSet;

use xrf_level::LevelFile;
use xrf_vfs::{XrayAssetType, XrayProbe};

use crate::plugins::levels::state::LevelTextureReference;

/// Resolves every texture a level's shader table names, base textures and lightmaps alike.
pub fn resolve_textures(level: &LevelFile, probe: &XrayProbe) -> Vec<LevelTextureReference> {
  let Some(shaders) = level.shaders.as_ref() else {
    return Vec::new();
  };

  let mut references: BTreeSet<&str> = BTreeSet::new();

  for entry in shaders.references() {
    for texture in &entry.textures {
      if !texture.is_empty() {
        references.insert(texture.as_str());
      }
    }
  }

  references
    .into_iter()
    .map(|reference| LevelTextureReference {
      logical_path: probe
        .resolve(XrayAssetType::Dds, reference)
        .ok()
        .and_then(|resolution| {
          resolution
            .get_asset()
            .map(|asset| asset.get_logical_path().as_str().to_owned())
        }),
      reference: reference.to_owned(),
    })
    .collect()
}
