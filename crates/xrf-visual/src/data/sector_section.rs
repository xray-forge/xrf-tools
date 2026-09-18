use serde::Serialize;

use crate::data::visual_section::VisualDrawRange;

/// One draw of a packed sector: the indices to draw, and the surface they are drawn with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorSection {
  /// Entry of the level's shader table every drawable in this section is dressed by.
  pub shader_id: u16,
  /// The engine shader that entry names, absent when the level carries no table.
  pub shader_name: Option<String>,
  /// The base texture that entry names, absent for the same reason.
  pub texture_name: Option<String>,
  /// The lightmaps the same entry names after it, which a lightmapped surface samples with its second uv set.
  pub lightmaps: Vec<String>,
  /// Drawables packed into this section, by their index in the visuals run.
  pub drawables: Vec<u32>,
  pub draw: VisualDrawRange,
}
