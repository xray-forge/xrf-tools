use serde::Serialize;

use crate::data::sector::sector_surface::SectorSurface;

/// A run of a sector's impostors dressed by one surface.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorImpostorGroup {
  pub surface: SectorSurface,
  /// The first impostor of the run.
  pub start: u32,
  pub count: u32,
}
