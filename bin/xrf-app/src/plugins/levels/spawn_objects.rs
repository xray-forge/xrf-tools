//! The spawned objects a level viewer draws: each object with a visual whose class is drawn, and the visuals they
//! stand as.

use std::collections::HashMap;

use xrf_spawn::AlifeObject;
use xrf_visual::VisualTransform;

use crate::plugins::levels::state::{LevelSpawn, LevelSpawnCategory, LevelSpawnObject, LevelSpawnObjectsDescription};

/// Every object of a level's spawn the viewer draws, in the order the spawn keeps them, each visual named once.
pub fn describe_spawn_objects(spawn: &LevelSpawn) -> LevelSpawnObjectsDescription {
  let mut visuals: Vec<String> = Vec::new();
  let mut indices: HashMap<&str, u32> = HashMap::new();
  let mut objects: Vec<LevelSpawnObject> = Vec::new();

  for (index, object) in spawn.objects.iter().enumerate() {
    let Some((category, name)) = get_drawn_visual(object) else {
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

  LevelSpawnObjectsDescription { visuals, objects }
}

/// What an object is drawn in and as, or `None` for one drawn as nothing: the one rule every reader of the drawn
/// objects goes by.
pub fn get_drawn_visual(object: &AlifeObject) -> Option<(LevelSpawnCategory, &str)> {
  Some((
    LevelSpawnCategory::of(&object.inherited)?,
    object.inherited.get_visual()?,
  ))
}
