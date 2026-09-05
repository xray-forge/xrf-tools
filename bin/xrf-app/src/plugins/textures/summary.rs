use rayon::prelude::*;
use serde::Serialize;
use xrf_material::{XrayBumpOutcome, XrayMaterialDescriptor, XrayMaterialResolver};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe, XrayRoots};

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;

/// Descriptors read per critical section, so interactive reads interleave with the sweep instead of queueing behind it.
const SWEEP_SLICE: usize = 512;

/// What a tree shows on a texture before anyone opens it, read from its descriptor alone.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBadges {
  /// Both halves of the declared pair resolved to the files the declaration names.
  pub is_bumped: bool,
  /// The bump shader path is taken but at least one half is a dummy or absent, so the surface is not what was authored.
  pub is_degraded: bool,
  /// The descriptor's texture type makes `LoadTHM` skip it whole, bump declaration included.
  pub is_engine_skipped: bool,
  /// A detail texture is named and one of the two flags that switch it on is set.
  pub is_detail_associated: bool,
  /// A `.thm` sits there and does not parse as one.
  pub is_unreadable: bool,
}

impl TextureBadges {
  pub fn of(material: &XrayMaterialDescriptor) -> Self {
    Self {
      is_bumped: material.outcome == XrayBumpOutcome::Bumped,
      is_degraded: material.outcome.is_degraded(),
      is_engine_skipped: material.is_engine_skipped(),
      is_detail_associated: material.is_detail_associated(),
      is_unreadable: material.is_unreadable(),
    }
  }
}

/// The two references a declaration binds, which is what folds a pair under the texture that declares it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBumpPair {
  pub bump: String,
  pub companion: String,
}

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

  /// Summarizes every descriptor a root set holds, taking the mounts in slices.
  ///
  /// Takes a handle on the mounts rather than a probe because this is the shape a blocking thread needs: the
  /// descriptors are listed once, then read slice by slice under the lock, and the summaries come back in listing
  /// order.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be planned or mounted.
  pub fn sweep_roots(mounts: &AssetMountState, roots: &XrayRoots) -> TauriResult<Vec<Self>> {
    let descriptors: Vec<XrayAsset> =
      mounts.with_probe(roots, |probe| probe.list_assets_of_type(XrayAssetType::Thm))?;
    let mut summaries: Vec<Self> = Vec::with_capacity(descriptors.len());

    for slice in descriptors.chunks(SWEEP_SLICE) {
      summaries.extend(mounts.with_probe(roots, |probe| Self::sweep(probe, slice))?);
    }

    Ok(summaries)
  }

  /// Summarizes the given descriptors the way `LoadTHM` reads them, in the order given.
  ///
  /// Descriptors are read in parallel on whatever pool the caller installed, because a corpus has thousands of them
  /// and each is a small parse behind a file or volume read. A descriptor outside `textures\` has no reference and is
  /// skipped, matching the catalog.
  pub fn sweep(probe: &XrayProbe, descriptors: &[XrayAsset]) -> Vec<Self> {
    descriptors
      .par_iter()
      .filter_map(|descriptor| {
        Some(Self::of(
          descriptor.to_reference()?,
          &XrayMaterialResolver::describe_descriptor(probe, descriptor),
        ))
      })
      .collect()
  }
}
