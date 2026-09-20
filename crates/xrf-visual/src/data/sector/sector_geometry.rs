use serde::Serialize;

use crate::data::visual::geometry::visual_section::VisualSection;

/// Where one packed mesh's attributes sit inside a sector's buffer, and what to draw from them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorGeometry {
  pub vertex_count: u32,
  pub index_count: u32,
  pub positions: VisualSection,
  pub normals: Option<VisualSection>,
  /// The authored tangent of every vertex, mirrored with the normal.
  pub tangents: Option<VisualSection>,
  /// The authored binormal of every vertex, mirrored with the normal.
  pub binormals: Option<VisualSection>,
  pub uvs: Option<VisualSection>,
  /// The lightmap coordinate of every vertex, for a surface xrLC lit from lightmaps.
  pub lightmap_uvs: Option<VisualSection>,
  /// The baked vertex colour of every vertex, as three floats in zero to one.
  pub colors: Option<VisualSection>,
  /// The hemisphere term of every vertex, which rides in the normal and is present with it.
  pub hemi: Option<VisualSection>,
  /// Every index, as 32-bit elements: a sector reaches past what sixteen bits address.
  pub indices: VisualSection,
}
