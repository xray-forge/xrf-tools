use serde::Serialize;
use xrf_editable_level::EditableShape;

use crate::scene::{SceneShapeElement, SceneShapeUsage};

/// The volume a shape encloses: its spheres and boxes, and what the shape is used for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneShape {
  pub usage: SceneShapeUsage,
  pub elements: Vec<SceneShapeElement>,
}

impl From<&EditableShape> for SceneShape {
  fn from(shape: &EditableShape) -> Self {
    Self {
      usage: SceneShapeUsage::of(shape.usage),
      elements: shape.elements.iter().map(SceneShapeElement::from).collect(),
    }
  }
}
