use serde::Serialize;
use xrf_math::Vector3d;

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_submesh::VisualSubmesh;

/// One transform in renderer space: three basis vectors and a translation.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualTransform {
  pub i: Vector3d,
  pub j: Vector3d,
  pub k: Vector3d,
  pub c: Vector3d,
}

/// One bone of a visual's skeleton, as a name and the name of its parent.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBone {
  pub name: String,
  pub parent: String,
  /// Index of the parent in this same list, or `None` for a root or a parent no bone carries.
  pub parent_index: Option<u32>,
  /// The bone's whole bind transform in model space, or `None` when the file carries no IK chunk.
  pub bind_transform: Option<VisualTransform>,
}

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
