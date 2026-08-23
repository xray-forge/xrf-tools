use serde::Serialize;
use xrf_db::Vector3d;

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_submesh::VisualSubmesh;

/// One bone of a visual's skeleton, as a name and the name of its parent.
///
/// A root bone carries an empty parent. Names rather than indices, because that is how OGF stores the
/// hierarchy and a tree can be rebuilt from them without further work.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBone {
  pub name: String,
  pub parent: String,
  /// Index of the parent in this same list, or `None` for a root or a parent no bone carries.
  pub parent_index: Option<u32>,
  /// Where the joint sits in the bind pose, in renderer space, or `None` when the file carries no IK chunk.
  pub bind_position: Option<Vector3d>,
}

/// Everything about a packed visual except the bytes themselves.
///
/// The counterpart of the geometry buffer: a consumer reads this first, then asks for the buffer and
/// builds views from the byte ranges each submesh carries. The reported total buffer length makes a
/// mismatched description and buffer detectable.
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
