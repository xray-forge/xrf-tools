use crate::data::visual_section::VisualSection;

/// Where a sector's attribute arrays landed once they were written into its buffer.
#[derive(Debug)]
pub(crate) struct SectorVertexSections {
  pub positions: VisualSection,
  pub normals: Option<VisualSection>,
  pub tangents: Option<VisualSection>,
  pub binormals: Option<VisualSection>,
  pub texture_coordinates: Option<VisualSection>,
  pub lightmap_coordinates: Option<VisualSection>,
  pub colors: Option<VisualSection>,
  pub hemi: Option<VisualSection>,
}
