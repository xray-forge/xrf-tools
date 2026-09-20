//! What a level's surfaces are dressed with, resolved once for the whole shader table.

use std::collections::BTreeSet;

use xrf_error::XrfResult;
use xrf_level::LevelFile;
use xrf_vfs::{XrayAssetRules, XrayAssetType, XrayProbe, XrayResolution};

use crate::plugins::levels::state::LevelTextureReference;

/// Resolves every texture a level's shader table names, base textures and lightmaps alike.
pub fn resolve_textures(level: &LevelFile, probe: &XrayProbe, directory: Option<&str>) -> Vec<LevelTextureReference> {
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
      logical_path: resolve_reference(probe, directory, reference),
      reference: reference.to_owned(),
    })
    .collect()
}

/// Locates one texture reference the way the engine's own loader does.
fn resolve_reference(probe: &XrayProbe, directory: Option<&str>, reference: &str) -> Option<String> {
  if let Some(beside) = directory
    .zip(XrayAssetType::Dds.get_rules())
    .and_then(|(directory, rules)| get_resolution_logical_path(probe.find(&beside_level(directory, &rules, reference))))
  {
    return Some(beside);
  }

  get_resolution_logical_path(probe.resolve(XrayAssetType::Dds, reference))
}

/// The path a reference names beside the level rather than below the shared texture tree.
fn beside_level(directory: &str, rules: &XrayAssetRules, reference: &str) -> String {
  format!("{directory}\\{}", rules.to_logical_path(reference))
}

/// The logical path a lookup landed on, treating a rejected reference as absent rather than failing the open.
fn get_resolution_logical_path(resolution: XrfResult<XrayResolution>) -> Option<String> {
  resolution.ok().and_then(|resolution| {
    resolution
      .get_asset()
      .map(|asset| asset.get_logical_path().as_str().to_owned())
  })
}
