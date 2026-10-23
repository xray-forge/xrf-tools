use serde::Serialize;
use xrf_vfs::{XrayAssetType, XrayResolution, XraySearch};

use crate::data::visual::visual_description::VisualDescription;
use crate::resolve::visual_motion_dependency::VisualMotionDependency;
use crate::resolve::visual_texture_dependency::VisualTextureDependency;

/// Everything a visual needs from outside itself, resolved.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDependencies {
  pub textures: Vec<VisualTextureDependency>,
  pub motions: Vec<VisualMotionDependency>,
}

impl VisualDependencies {
  /// The texture the engine substitutes for a reference it cannot find.
  pub const MISSING_TEXTURE_REFERENCE: &'static str = "ed\\ed_not_existing_texture";

  /// Resolves every reference a visual declares, in the order the search searches.
  pub fn resolve(description: &VisualDescription, search: &XraySearch) -> Self {
    Self {
      textures: description
        .submeshes
        .iter()
        .filter_map(|submesh| {
          submesh.texture_name.as_ref().map(|reference| VisualTextureDependency {
            submesh_index: submesh.index,
            reference: reference.clone(),
            resolution: Self::resolve_texture(search, reference),
          })
        })
        .collect(),
      motions: description
        .motion_refs
        .iter()
        .map(|reference| VisualMotionDependency {
          reference: reference.clone(),
          resolution: Self::resolve_motion(search, reference),
        })
        .collect(),
    }
  }

  /// The outcome for one submesh, by the index the submesh reports.
  pub fn find_texture(&self, submesh_index: u32) -> Option<&VisualTextureDependency> {
    self
      .textures
      .iter()
      .find(|texture| texture.submesh_index == submesh_index)
  }

  /// Resolves a texture reference, substituting the engine's dummy the way the renderer does.
  fn resolve_texture(search: &XraySearch, reference: &str) -> XrayResolution {
    Self::tolerate(search.resolve_with_fallback(XrayAssetType::Dds, reference, Self::MISSING_TEXTURE_REFERENCE))
  }

  /// Resolves a motion reference, which has no substitute: nothing stands in for an absent animation set.
  fn resolve_motion(search: &XraySearch, reference: &str) -> XrayResolution {
    Self::tolerate(search.resolve(XrayAssetType::Omf, reference))
  }

  fn tolerate(resolution: xrf_error::XrfResult<XrayResolution>) -> XrayResolution {
    resolution.unwrap_or_else(|error| XrayResolution::Rejected {
      reason: error.to_string(),
    })
  }
}
