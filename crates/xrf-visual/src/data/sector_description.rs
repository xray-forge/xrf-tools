use serde::Serialize;

use crate::data::sector_section::SectorSection;
use crate::data::sector_skip::SectorSkip;
use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_section::VisualSection;

/// Everything about a packed sector except the bytes themselves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDescription {
  /// The sector packed, by its index in the sectors chunk.
  pub sector: u32,
  pub vertex_count: u32,
  pub index_count: u32,
  pub positions: VisualSection,
  pub normals: Option<VisualSection>,
  /// The authored tangent of every vertex, mirrored with the normal.
  pub tangents: Option<VisualSection>,
  /// The authored binormal of every vertex, mirrored with the normal.
  pub binormals: Option<VisualSection>,
  pub texture_coordinates: Option<VisualSection>,
  /// The lightmap coordinate of every vertex, for a sector xrLC lit from lightmaps.
  pub lightmap_coordinates: Option<VisualSection>,
  /// The baked vertex colour of every vertex, as three floats in zero to one.
  pub colors: Option<VisualSection>,
  /// The hemisphere term of every vertex, which rides in the normal and is present with it.
  pub hemi: Option<VisualSection>,
  /// Every index of the sector, as 32-bit elements, laid out section by section.
  pub indices: VisualSection,
  pub sections: Vec<SectorSection>,
  /// Drawables that produced no geometry, which is none for every level measured.
  pub skipped: Vec<SectorSkip>,
  /// Extent the packed vertices span, absent when the sector packed none.
  pub bounds: Option<VisualBounds>,
  /// Vertex ranges more than one drawable named, and so packed once rather than once each.
  pub shared_ranges: u32,
  /// Vertices the sector would hold if every drawable brought its own copy of the range it names, which is what the
  /// sharing saves.
  pub unshared_vertex_count: u32,
  pub buffer_length: u32,
}
