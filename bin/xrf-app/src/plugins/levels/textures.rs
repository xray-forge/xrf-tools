//! What a level's surfaces are dressed with, resolved once for the whole shader table.

use std::collections::{BTreeMap, BTreeSet};

use xrf_level::LevelFile;
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayProbe, XrayResolution};

use crate::plugins::levels::state::LevelTextureReference;

/// The shader an impostor is drawn with, whose atlas is bound beside a companion of its own.
const IMPOSTOR_SHADER: &str = "details\\lod";

/// What `details_lod.s` appends to the atlas for its `s_hemi`: a normal in colour, the hemisphere term in alpha.
const IMPOSTOR_COMPANION_SUFFIX: &str = "_nm";

/// Resolves every texture a level's surfaces bind: base textures, lightmaps, detail textures, bump pairs, and the
/// companion an impostor's atlas is bound with.
pub fn resolve_level_textures(
  level: &LevelFile,
  surfaces: &[XraySurfaceDescriptor],
  probe: &XrayProbe,
  scope: &XrayTextureScope,
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

  resolve_surface_textures(references, surfaces, probe, scope)
}

/// Resolves the textures a run of surfaces binds beside those named for it: each one's detail, the files its script
/// binds, and its bump pair, found as its descriptor found it.
pub fn resolve_surface_textures(
  mut references: BTreeSet<String>,
  surfaces: &[XraySurfaceDescriptor],
  probe: &XrayProbe,
  scope: &XrayTextureScope,
) -> Vec<LevelTextureReference> {
  for detail in surfaces.iter().filter_map(|surface| surface.detail.as_ref()) {
    references.insert(detail.reference.clone());
  }

  // A scripted surface binds files of its own, which the shader table never names: water's normal map and foam.
  for sampler in surfaces.iter().flat_map(|surface| &surface.samplers) {
    references.insert(sampler.texture.clone());
  }

  // A bump pair is found as the descriptor that declared it found it, beside the level first and the engine's dummy
  // where its file is absent.
  let bumps: BTreeMap<String, Option<String>> = surfaces
    .iter()
    .flat_map(|surface| {
      surface
        .bump
        .iter()
        .chain(surface.detail.as_ref().and_then(|detail| detail.bump.as_ref()))
    })
    .flat_map(|bump| [&bump.bump, &bump.companion])
    .map(|input| (input.reference.clone(), get_located_path(&input.resolution)))
    .collect();

  let located: Vec<LevelTextureReference> = references
    .into_iter()
    .filter(|reference| !bumps.contains_key(reference))
    .map(|reference| LevelTextureReference {
      logical_path: resolve_reference(probe, scope, &reference),
      reference,
    })
    .collect();

  located
    .into_iter()
    .chain(
      bumps
        .into_iter()
        .map(|(reference, logical_path)| LevelTextureReference {
          logical_path,
          reference,
        }),
    )
    .collect()
}

/// Locates one texture reference the way the engine's own loader does, treating a rejected reference as absent rather
/// than failing the open.
pub fn resolve_reference(probe: &XrayProbe, scope: &XrayTextureScope, reference: &str) -> Option<String> {
  scope
    .resolve_texture(probe, reference)
    .ok()
    .as_ref()
    .and_then(get_located_path)
}

/// The logical path a lookup landed on.
fn get_located_path(resolution: &XrayResolution) -> Option<String> {
  resolution
    .get_asset()
    .map(|asset| asset.get_logical_path().as_str().to_owned())
}
