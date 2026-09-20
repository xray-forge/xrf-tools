use serde::Serialize;

use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;
use crate::data::visual::geometry::visual_section::VisualSection;
use crate::data::visual::geometry::visual_skin::VisualSkin;

/// Where one submesh's attributes sit inside the geometry buffer, and what to draw from them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualGeometry {
  pub vertex_count: u32,
  pub index_count: u32,
  pub positions: VisualSection,
  pub normals: VisualSection,
  /// The authored tangent of every vertex, mirrored with the normal.
  pub tangents: VisualSection,
  /// The authored binormal of every vertex, mirrored with the normal; see [`Self::tangents`].
  pub binormals: VisualSection,
  pub uvs: VisualSection,
  pub indices: VisualSection,
  /// Skinning links, or `None` for geometry that carries none and is therefore drawn as it is stored.
  pub skin: Option<VisualSkin>,
  /// Every range a consumer may draw, finest first, and never empty.
  pub detail_levels: Vec<VisualDrawRange>,
  pub bounds: VisualBounds,
}

impl VisualGeometry {
  /// The range drawn unless a consumer picks another level: the finest one.
  ///
  /// # Panics
  ///
  /// Never in practice. [`Self::detail_levels`] is non-empty by construction - a submesh whose finest level does
  /// not validate is reported as skipped rather than packed - and this states that invariant where it is relied on.
  pub fn get_default_level(&self) -> VisualDrawRange {
    self.detail_levels[0]
  }
}
