use serde::Serialize;

use crate::data::sector::sector_geometry::SectorGeometry;
use crate::data::sector::sector_impostors::SectorImpostors;
use crate::data::sector::sector_instance_group::SectorInstanceGroup;
use crate::data::sector::sector_section::SectorSection;
use crate::data::sector::sector_skip::SectorSkip;
use crate::data::visual::bounds::visual_bounds::VisualBounds;

/// Everything about a packed sector except the bytes themselves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDescription {
  /// The sector packed, by its index in the sectors chunk.
  pub sector: u32,
  /// Everything the level bakes in place, packed onto one vertex array and drawn section by section.
  pub geometry: SectorGeometry,
  pub sections: Vec<SectorSection>,
  /// Meshes the sector draws many times over, each packed once with the places it stands.
  pub instances: Vec<SectorInstanceGroup>,
  /// What its clumps of trees draw as from far enough away, absent for a sector with no `MT_LOD` visual.
  pub impostors: Option<SectorImpostors>,
  /// Drawables that produced no geometry, which is none for every level measured.
  pub skipped: Vec<SectorSkip>,
  /// Extent the packed vertices span, absent when the sector packed none.
  pub bounds: Option<VisualBounds>,
  pub buffer_length: u32,
}
