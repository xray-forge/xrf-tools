use serde::Serialize;
use xrf_vfs::XrayResolution;

use crate::data::xray_bump_fallback::XrayBumpFallback;

/// What the renderer ends up drawing for a material, mirroring `Texture.cpp`.
///
/// Ordered from best to worst so the outcome of a pair is the worse of its two inputs: a real bump over a dummy
/// companion is `Dummy`, and a dummy bump beside a companion that fell to the not-existing texture is `Missing`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayBumpOutcome {
  /// No usable declaration, so the flat shader variant is selected and no bump input is bound.
  Flat,
  /// Both inputs resolved to the files the declaration names.
  Bumped,
  /// The bump shader variant is selected and at least one input is the engine's flat dummy, because the declared name
  /// contains `_bump` and its file is absent. The surface renders flat while paying the bump path, and the engine logs
  /// `! Fallback to default bump map`.
  Dummy,
  /// At least one input is absent and has no dummy: its name lacks `_bump`, so `ed\ed_not_existing_texture` is bound
  /// in its place, or nothing at all when even that is missing.
  Missing,
}

impl XrayBumpOutcome {
  /// What one bound input came to.
  pub fn of_input(resolution: &XrayResolution) -> Self {
    match resolution {
      XrayResolution::Resolved { .. } => Self::Bumped,
      XrayResolution::Substituted { fallback, .. }
        if XrayBumpFallback::of_reference(fallback).is_some_and(XrayBumpFallback::is_dummy) =>
      {
        Self::Dummy
      }
      XrayResolution::Substituted { .. }
      | XrayResolution::Missing { .. }
      | XrayResolution::NoScope
      | XrayResolution::Rejected { .. } => Self::Missing,
    }
  }

  /// What a pair comes to: the worse of its two inputs.
  pub fn of_pair(bump: &XrayResolution, companion: &XrayResolution) -> Self {
    Self::of_input(bump).max(Self::of_input(companion))
  }

  /// Whether the bump shader variant is selected, whatever ends up bound to it.
  pub fn is_bump_path(self) -> bool {
    self != Self::Flat
  }

  /// Whether the surface the engine draws is not the surface the author declared.
  pub fn is_degraded(self) -> bool {
    matches!(self, Self::Dummy | Self::Missing)
  }
}
