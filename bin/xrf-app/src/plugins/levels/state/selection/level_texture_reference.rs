use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayResolution, XraySearch};

/// One texture a level's shader table names, and what it came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTextureReference {
  /// The reference as the shader table spells it, which is what a surface names.
  pub reference: String,
  /// What it resolved to, or `None` for a reference the roots hold nothing for.
  pub logical_path: Option<String>,
}

impl LevelTextureReference {
  /// A reference as `scope` finds it through `search`, as the engine's loader finds it.
  pub fn locate(scope: &XrayTextureScope, search: &XraySearch, reference: String) -> Self {
    Self {
      logical_path: scope.locate_texture(search, &reference),
      reference,
    }
  }

  /// The textures a run of surfaces binds: `references`, those named for it, then each surface's detail, the files its
  /// script binds, and its bump pair, found as its descriptor found it.
  pub fn resolve_bound(
    mut references: BTreeSet<String>,
    surfaces: &[XraySurfaceDescriptor],
    search: &XraySearch,
    scope: &XrayTextureScope,
  ) -> Vec<Self> {
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
      .map(|input| (input.reference.clone(), Self::get_located_path(&input.resolution)))
      .collect();
    let located: Vec<Self> = references
      .into_iter()
      .filter(|reference| !bumps.contains_key(reference))
      .map(|reference| Self::locate(scope, search, reference))
      .collect();

    located
      .into_iter()
      .chain(bumps.into_iter().map(|(reference, logical_path)| Self {
        logical_path,
        reference,
      }))
      .collect()
  }

  /// The logical path a lookup landed on.
  fn get_located_path(resolution: &XrayResolution) -> Option<String> {
    resolution
      .get_asset()
      .map(|asset| asset.get_logical_path().as_str().to_owned())
  }
}
