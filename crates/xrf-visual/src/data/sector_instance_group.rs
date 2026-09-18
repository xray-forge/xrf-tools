use serde::Serialize;

use crate::data::visual_section::VisualSection;

/// One mesh a sector draws many times, packed once with the places it stands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorInstanceGroup {
  /// Entry of the level's shader table every instance is dressed by.
  pub shader_id: u16,
  pub shader_name: Option<String>,
  pub texture_name: Option<String>,
  pub lightmaps: Vec<String>,
  /// The drawables this group stands in for, by their index in the visuals run.
  pub drawables: Vec<u32>,
  pub vertex_count: u32,
  pub index_count: u32,
  pub instance_count: u32,
  pub positions: VisualSection,
  pub normals: Option<VisualSection>,
  pub tangents: Option<VisualSection>,
  pub binormals: Option<VisualSection>,
  pub texture_coordinates: Option<VisualSection>,
  pub lightmap_coordinates: Option<VisualSection>,
  pub colors: Option<VisualSection>,
  pub hemi: Option<VisualSection>,
  /// The mesh's own indices, counting from its own first vertex.
  pub indices: VisualSection,
  /// Sixteen floats for each instance, exactly as the engine stores a matrix.
  pub transforms: VisualSection,
}

impl SectorInstanceGroup {
  /// Floats one instance's transform occupies.
  pub const FLOATS_PER_INSTANCE: usize = 16;
}
