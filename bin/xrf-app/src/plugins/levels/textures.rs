//! What a level's surfaces are dressed with, resolved once for the whole shader table.

use std::collections::BTreeSet;

use xrf_error::XrfResult;
use xrf_level::LevelFile;
use xrf_material::XraySurfaceDescriptor;
use xrf_vfs::{XrayAssetRules, XrayAssetType, XrayProbe, XrayResolution};

use crate::plugins::levels::state::LevelTextureReference;

/// The shader an impostor is drawn with, whose atlas is bound beside a companion of its own.
const IMPOSTOR_SHADER: &str = "details\\lod";

/// What `details_lod.s` appends to the atlas for its `s_hemi`: a normal in colour, the hemisphere term in alpha.
const IMPOSTOR_COMPANION_SUFFIX: &str = "_nm";

/// Resolves every texture a level's surfaces bind: base textures, lightmaps, detail textures, and the companion an
/// impostor's atlas is bound with.
pub fn resolve_textures(
  level: &LevelFile,
  surfaces: &[XraySurfaceDescriptor],
  probe: &XrayProbe,
  directory: Option<&str>,
) -> Vec<LevelTextureReference> {
  let mut references: BTreeSet<String> = BTreeSet::new();

  if let Some(shaders) = level.shaders.as_ref() {
    for entry in shaders.references() {
      for texture in entry.textures.iter().filter(|texture| !texture.is_empty()) {
        references.insert(texture.clone());
      }

      if entry.shader.eq_ignore_ascii_case(IMPOSTOR_SHADER)
        && let Some(atlas) = entry.textures.first().filter(|texture| !texture.is_empty())
      {
        references.insert(format!("{atlas}{IMPOSTOR_COMPANION_SUFFIX}"));
      }
    }
  }

  for detail in surfaces.iter().filter_map(|surface| surface.detail.as_ref()) {
    references.insert(detail.reference.clone());
  }

  references
    .into_iter()
    .map(|reference| LevelTextureReference {
      logical_path: resolve_reference(probe, directory, &reference),
      reference,
    })
    .collect()
}

/// Locates one texture reference the way the engine's own loader does.
pub fn resolve_reference(probe: &XrayProbe, directory: Option<&str>, reference: &str) -> Option<String> {
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
