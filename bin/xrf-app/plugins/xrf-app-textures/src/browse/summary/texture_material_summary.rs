use rayon::prelude::*;
use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_material::{XrayMaterialDescriptor, XrayMaterialResolver};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayRoots, XraySearch};

use crate::browse::summary::{TextureBadges, TextureBumpPair};

/// One descriptor's contribution to the tree: its badges, and the pair it names so both halves fold under it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureMaterialSummary {
  /// The reference of the texture the descriptor describes.
  pub reference: String,
  /// The pair the engine will try to bind, when the declaration is one it reads.
  pub bump: Option<TextureBumpPair>,
  pub badges: TextureBadges,
}

impl TextureMaterialSummary {
  /// What a material's descriptor makes of the texture `reference` names.
  pub fn of(reference: String, material: &XrayMaterialDescriptor) -> Self {
    Self {
      reference,
      bump: material.declared_bump_pair().map(|(bump, companion)| TextureBumpPair {
        bump: bump.to_owned(),
        companion: companion.to_owned(),
      }),
      badges: TextureBadges::of(material),
    }
  }

  /// Summarizes every descriptor a root set holds, through the one snapshot they were listed from.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be planned or mounted.
  pub fn sweep_roots(assets: &AssetMountState, roots: &XrayRoots) -> TauriResult<Vec<Self>> {
    let search: XraySearch = assets.search(roots)?;

    Ok(Self::sweep(&search, &search.list_assets(XrayAssetType::Thm)))
  }

  /// Summarizes the given descriptors the way `LoadTHM` reads them, in the order given.
  pub fn sweep(search: &XraySearch, descriptors: &[XrayAsset]) -> Vec<Self> {
    descriptors
      .par_iter()
      .filter_map(|descriptor| {
        Some(Self::of(
          descriptor.to_reference()?,
          &XrayMaterialResolver::describe_descriptor(search, descriptor),
        ))
      })
      .collect()
  }
}
