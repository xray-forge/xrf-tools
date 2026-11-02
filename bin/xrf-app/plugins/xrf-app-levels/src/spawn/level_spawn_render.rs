use std::collections::HashMap;

use xrf_renderer::{RenderLevelSpawn, RenderSpawnObject};
use xrf_visual::VisualTransform;

use crate::parts::LevelSpawn;
use crate::spawn::LevelSpawnCategory;

impl From<&LevelSpawn> for RenderLevelSpawn {
  /// The objects of a level's spawn the renderer draws, in the spawn's order, each visual named once in the order first
  /// stood.
  fn from(spawn: &LevelSpawn) -> Self {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let mut objects: Vec<RenderSpawnObject> = Vec::new();

    for (index, object) in spawn.objects.iter().enumerate() {
      let Some((category, name)) = LevelSpawnCategory::get_drawn_visual(object) else {
        continue;
      };
      let visual: u32 = *indices.entry(name).or_insert_with(|| {
        visuals.push(name.to_owned());

        (visuals.len() - 1) as u32
      });

      objects.push(RenderSpawnObject {
        index: index as u32,
        category: category.into(),
        is_released: spawn.get_release(index).is_some(),
        visual,
        transform: VisualTransform::of_spawn(&object.position, &object.direction).to_matrix(),
      });
    }

    Self {
      objects,
      visuals,
      detail: 0.0,
    }
  }
}
