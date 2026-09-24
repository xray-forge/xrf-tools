use serde::Serialize;

use crate::data::sector::sector_impostor_group::SectorImpostorGroup;
use crate::data::visual::geometry::visual_section::VisualSection;

/// The impostors of a sector's `MT_LOD` visuals: what the engine draws in place of a clump of trees seen from far
/// enough away, each eight facets looking at it from eight sides, in renderer space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorImpostors {
  pub count: u32,
  /// Runs of impostors a surface each, in impostor order.
  pub groups: Vec<SectorImpostorGroup>,
  /// Four floats an impostor: its visual's sphere, centre then radius.
  pub spheres: VisualSection,
  /// One float an impostor: `FLOD::lod_factor`, what its sphere's screen area is scaled by.
  pub factors: VisualSection,
  /// Eight floats for each of an impostor's 32 corners, facet by facet: the position, the atlas `u` and `v`, then the
  /// hemisphere and sun terms as the bytes it stores them over 255, then nothing.
  pub corners: VisualSection,
  /// Four floats for each of an impostor's eight facets: its normal, then nothing.
  pub normals: VisualSection,
}

impl SectorImpostors {
  /// Corners one impostor has: eight facets of four.
  pub const CORNERS: usize = 32;

  /// Floats one corner takes.
  pub const FLOATS_PER_CORNER: usize = 8;
}
