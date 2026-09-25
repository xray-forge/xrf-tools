use serde::Serialize;

use crate::data::visual::geometry::visual_section::VisualSection;

/// One detail model of a level's library, packed for the renderer to plant.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailsModel {
  /// The shader and texture it is dressed with, as the library names them.
  pub shader: String,
  pub texture: String,
  /// Whether the wind moves it: no `DO_NO_WAVING` flag.
  pub is_waving: bool,
  /// The scale range it is planted at, before the engine narrows it to half the least and nine tenths the most.
  pub min_scale: f32,
  pub max_scale: f32,
  /// Its bounding box's height, which a waving vertex's share of the sway is measured against.
  pub height: f32,
  /// The radius of the sphere around its bounding box, which its distance cull is measured by.
  pub radius: f32,
  pub vertex_count: u32,
  pub index_count: u32,
  /// Three floats a vertex, in renderer space.
  pub positions: VisualSection,
  /// Two floats a vertex.
  pub uvs: VisualSection,
  /// Sixteen-bit indices, wound for renderer space.
  pub indices: VisualSection,
}
