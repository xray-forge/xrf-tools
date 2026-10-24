use std::collections::HashMap;

use serde::Serialize;
use xrf_visual::VisualTransform;

use crate::state::{LevelSpawn, LevelSpawnCategory, LevelSpawnObject};

/// The spawned objects the viewer draws, and the visuals they stand as, each named once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectsDescription {
  pub visuals: Vec<String>,
  pub objects: Vec<LevelSpawnObject>,
}

impl LevelSpawnObjectsDescription {
  /// Every object of a level's spawn the viewer draws, in the order the spawn keeps them, each visual named once.
  pub fn describe(spawn: &LevelSpawn) -> Self {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let mut objects: Vec<LevelSpawnObject> = Vec::new();

    for (index, object) in spawn.objects.iter().enumerate() {
      let Some((category, name)) = LevelSpawnCategory::get_drawn_visual(object) else {
        continue;
      };

      let visual: u32 = *indices.entry(name).or_insert_with(|| {
        visuals.push(name.to_owned());

        (visuals.len() - 1) as u32
      });

      objects.push(LevelSpawnObject {
        category,
        clsid: object.clsid.clone(),
        index: index as u32,
        name: object.name.clone(),
        release: spawn.get_release(index),
        section: object.section.clone(),
        story_id: object.inherited.get_abstract().and_then(|it| it.get_story_id()),
        transform: VisualTransform::of_spawn(&object.position, &object.direction),
        visual,
      });
    }

    Self { visuals, objects }
  }
}
