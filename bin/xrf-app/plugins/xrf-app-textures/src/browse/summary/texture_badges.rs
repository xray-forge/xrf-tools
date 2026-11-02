use serde::Serialize;
use xrf_material::{XrayBumpOutcome, XrayMaterialDescriptor};

/// What a tree shows on a texture before anyone opens it, read from its descriptor alone.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBadges {
  /// Both halves of the declared pair resolved to the files the declaration names.
  pub is_bumped: bool,
  /// The bump shader path is taken but a half is a dummy or absent, so the surface is not what was authored.
  pub is_degraded: bool,
  /// The descriptor's texture type makes `LoadTHM` skip it whole, bump declaration included.
  pub is_engine_skipped: bool,
  /// A detail texture is named and one of the two flags that switch it on is set.
  pub is_detail_associated: bool,
  /// A `.thm` sits there and does not parse as one.
  pub is_unreadable: bool,
}

impl TextureBadges {
  /// The badges a material's descriptor earns.
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
