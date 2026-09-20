use serde::Serialize;

use crate::data::visual_bone::VisualBone;
use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_submesh::VisualSubmesh;

/// Everything about a packed visual except the bytes themselves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDescription {
  pub version: u8,
  pub model_type: u8,
  pub model_type_label: String,
  pub shader_id: u16,
  /// Source object the OGF was built from, when the file records one.
  pub source_file: Option<String>,
  /// Extent the header declares, converted into three.js space for comparison with the computed extent.
  pub declared_bounds: VisualBounds,
  /// Extent the packed geometry actually spans, absent when no submesh produced any.
  pub computed_bounds: Option<VisualBounds>,
  pub submeshes: Vec<VisualSubmesh>,
  pub bones: Vec<VisualBone>,
  /// Logical paths of the omf files this visual animates from.
  pub motion_refs: Vec<String>,
  /// Names of motions stored inside the visual itself, for a self animated model.
  pub embedded_motions: Vec<String>,
  pub buffer_length: u32,
}
