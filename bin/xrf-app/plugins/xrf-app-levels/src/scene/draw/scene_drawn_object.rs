use xrf_editable_level::{EditableObjectTransform, EditableSceneObject};
use xrf_extension::XrayExtension;
use xrf_math::Affine3d;
use xrf_visual::VisualTransform;

use crate::scene::SceneObjectId;

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

  /// Where it stands as the renderer places its model, sixteen floats column by column: the world transform
  /// `CCustomObject::OnUpdateTransform` composes, which the level build places its meshes' points by, mirrored through
  /// `z` into renderer space as the model's vertices are.
  pub(crate) fn get_render_transform(&self) -> [f32; 16] {
    let placement: Affine3d = self.transform.to_matrix().to_affine();

    VisualTransform {
      i: placement.x_axis,
      j: placement.y_axis,
      k: placement.z_axis,
      c: placement.translation,
    }
    .mirrored()
    .to_matrix()
  }
}
