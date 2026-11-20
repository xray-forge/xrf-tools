use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableSpawnPointKind, EditableSpawnPointObject, EditableToolObjects};
use xrf_rawdata::LevelSpawnDataFields;

use crate::scene::{SceneObjectId, SceneSpawnModel, SceneToolKind, SelectedScene};
use crate::visuals::SpawnVisualReader;

/// The spawn points a scene draws as their entities' models, as the SDK's editor does: each whose packet reads and
/// names a visual the scene's game data holds. The rest keep their icon.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SceneSpawnModels {
  /// Every spawn point drawn as a model, in spawn point order.
  pub objects: Vec<SceneSpawnModel>,
}

impl SceneSpawnModels {
  /// The spawn points of `scene`, indexed as the scene's objects are, whose packet names a visual `reader` finds. Empty
  /// where the scene has no game data or its spawn points do not read; the scene's objects report the latter.
  pub(crate) fn read(scene: &SelectedScene, reader: &SpawnVisualReader) -> Self {
    profiling::scope!("scene spawn models");

    if scene.game.roots.is_empty() {
      return Self::default();
    }

    let Ok(read) = EditableToolObjects::<EditableSpawnPointObject>::read(&scene.scene, scene.guid) else {
      return Self::default();
    };
    let mut models: Self = Self::default();

    for (index, point) in read.iter_indexed() {
      let EditableSpawnPointKind::Entity { data, .. } = &point.kind else {
        continue;
      };
      // A packet that does not read is the build's to refuse; the spawn point keeps its icon here.
      let Ok(Some(visual)) = LevelSpawnDataFields::read_visual::<XRayByteOrder>(data) else {
        continue;
      };

      if reader.is_found(&visual) {
        models.objects.push(SceneSpawnModel {
          id: SceneObjectId {
            tool: SceneToolKind::Spawn,
            index,
          },
          visual,
          transform: point.object.transform,
        });
      }
    }

    models
  }

  /// The spawn point at `index`, if it is drawn as a model.
  pub(crate) fn find(&self, index: u32) -> Option<&SceneSpawnModel> {
    self
      .objects
      .binary_search_by_key(&index, |model| model.id.index)
      .ok()
      .map(|found| &self.objects[found])
  }

  /// The first spawn point drawn as the renderer's model `name`; every other drawn as it shares that model.
  pub(crate) fn find_model(&self, name: &str) -> Option<&SceneSpawnModel> {
    self.objects.iter().find(|model| model.get_model_name() == name)
  }
}
