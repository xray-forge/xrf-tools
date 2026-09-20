use serde::Serialize;

use crate::data::visual::skeleton::visual_transform::VisualTransform;

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
