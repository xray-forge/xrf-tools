use xrf_editable_level::{EditableObjectTransform, EditableSceneObject};
use xrf_extension::XrayExtension;

use crate::scene::{SceneObjectId, SceneRenderSpace};

/// One scene object the scene draws: the library object it places, found in the objects roots, and where it stands.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneDrawnObject {
  pub id: SceneObjectId,
  /// The library object it places, which names the model it draws as.
  pub reference: String,
  pub transform: EditableObjectTransform,
  /// Where its `.object` was found below the objects roots.
  pub logical_path: String,
}

impl SceneDrawnObject {
  /// Where the `.object` a scene object places is looked for below the objects roots: its reference, the extension
  /// added.
  pub(crate) fn format_object_path(object: &EditableSceneObject) -> String {
    format!("{}.{}", object.reference, XrayExtension::Object.as_str())
  }

  /// Where it stands as the renderer places its model, sixteen floats column by column: its transform in renderer
  /// space, as the model's vertices are.
  pub(crate) fn get_render_transform(&self) -> [f32; 16] {
    SceneRenderSpace::place(&self.transform)
  }
}
