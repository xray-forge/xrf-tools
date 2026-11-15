use xrf_editable_level::EditableSceneObject;
use xrf_rawdata::LevelSceneToolObjects;
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneDrawProblem, SceneDrawProblemKind, SceneDrawnObject, SceneObjectId, SceneToolKind, SelectedScene,
};

/// The scene objects a scene draws, each placing a library object found in its objects roots, and those it cannot,
/// each a problem.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SceneDrawnObjects {
  /// Every scene object whose `.object` was found, in the scene's order of its objects.
  pub objects: Vec<SceneDrawnObject>,
  /// Every scene object whose `.object` was not, in the same order.
  pub problems: Vec<SceneDrawProblem>,
}

impl SceneDrawnObjects {
  /// The scene objects of `scene`'s scene object tool as the editor loads them, the groups' members first, each named
  /// by its place among them as the scene's objects are, and its `.object` looked for through `objects`, a search of
  /// the scene's objects roots; none where the tool's objects do not read, which the scene's objects report.
  pub(crate) fn read(scene: &SelectedScene, objects: &XraySearch) -> Self {
    let Ok(read) = LevelSceneToolObjects::<EditableSceneObject>::read(&scene.scene, scene.guid) else {
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

  /// The first scene object drawn as the library object `reference`, which every other placing it shares a model with.
  pub(crate) fn find(&self, reference: &str) -> Option<&SceneDrawnObject> {
    self.objects.iter().find(|object| object.reference == reference)
  }
}
