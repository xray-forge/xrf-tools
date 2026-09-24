use serde::Serialize;

use crate::data::visual::geometry::visual_section::VisualSection;

/// Where one packed mesh's attributes sit inside a sector's buffer, and what to draw from them: positions as floats
/// in renderer space, and the rest in the 32-byte vertex xrLC wrote, byte for byte but for a direction's z.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorGeometry {
  pub vertex_count: u32,
  pub index_count: u32,
  /// Three floats a vertex, in renderer space.
  pub positions: VisualSection,
  /// Four bytes a vertex: the normal's z, y and x as `D3DCOLOR` stores them, z negated into renderer space, then the
  /// hemisphere term.
  pub normals: Option<VisualSection>,
  /// Four bytes a vertex the same way: the authored tangent, then the low byte of the base `u`.
  pub tangents: Option<VisualSection>,
  /// Four bytes a vertex the same way: the authored binormal, then the low byte of the base `v`.
  pub binormals: Option<VisualSection>,
  /// The base coordinate as xrLC quantised it, as many shorts a vertex as its components say.
  pub uvs: Option<VisualSection>,
  /// Shorts a base coordinate takes a vertex: two (`SHORT2`, over 1024 with its low bytes), or four for a tree
  /// (`SHORT4`, over 2048, then its wind terms). Zero where there are no base coordinates.
  pub uv_components: u32,
  /// The lightmap coordinate: two shorts a vertex, over 32768.
  pub lightmap_uvs: Option<VisualSection>,
  /// Every index, as 32-bit elements: a sector reaches past what sixteen bits address.
  pub indices: VisualSection,
}
