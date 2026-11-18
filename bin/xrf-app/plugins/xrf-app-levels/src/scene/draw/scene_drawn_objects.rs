use xrf_editable_level::{EditableSceneObject, EditableToolObjects};
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneDrawProblem, SceneDrawProblemKind, SceneDrawnObject, SceneObjectId, SceneToolKind, SelectedScene,
};

/// The scene objects whose library object was found in the objects roots, and a problem for each one whose was not.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SceneDrawnObjects {
  /// Every scene object whose `.object` was found, in the scene's order of its objects.
  pub objects: Vec<SceneDrawnObject>,
  /// Every scene object whose `.object` was not found, in the same order.
  pub problems: Vec<SceneDrawProblem>,
}

impl SceneDrawnObjects {
  /// Looks up each object of the scene object tool, as the editor loads them with group members first and indexed as
  /// the scene's objects are, through `objects`. Empty where the tool's objects do not read; the scene's objects report
  /// that.
  pub(crate) fn read(scene: &SelectedScene, objects: &XraySearch) -> Self {
    let Ok(read) = EditableToolObjects::<EditableSceneObject>::read(&scene.scene, scene.guid) else {
      return Self::default();
    };
    let mut drawn: Self = Self::default();

    for (object, index) in read.into_part().objects.into_iter().zip(0u32..) {
      let id: SceneObjectId = SceneObjectId {
        tool: SceneToolKind::SceneObject,
        index,
      };
      let path: String = SceneDrawnObject::format_object_path(&object);

      match objects.find(&path) {
        Ok(Some(asset)) => drawn.objects.push(SceneDrawnObject {
          id,
          reference: object.reference,
          transform: object.object.transform,
          logical_path: asset.get_logical_path().as_str().to_owned(),
        }),
        Ok(None) => drawn.problems.push(SceneDrawProblem {
          object: id,
          reference: object.reference,
          kind: SceneDrawProblemKind::Missing,
          message: format!("Library object '{path}' is not in the scene's objects roots"),
        }),
        Err(error) => drawn.problems.push(SceneDrawProblem {
          object: id,
          reference: object.reference,
          kind: SceneDrawProblemKind::Missing,
          message: format!("Library object '{path}' cannot be looked for: {error}"),
        }),
      }
    }

    drawn
  }

  /// The first scene object placing the library object `reference`; every other placing it shares its model.
  pub(crate) fn find(&self, reference: &str) -> Option<&SceneDrawnObject> {
    self.objects.iter().find(|object| object.reference == reference)
  }
}
