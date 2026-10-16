use xrf_renderer::{RenderLevelSpawn, RenderSpawnObject};

/// A level's spawned objects with each visual's own listed, so placing a model reads only the objects standing as it.
pub struct IndexedSpawn {
  pub spawn: RenderLevelSpawn,
  /// Each visual's objects, by their place in the spawn's objects.
  by_visual: Vec<Vec<usize>>,
}

impl IndexedSpawn {
  pub fn new(spawn: RenderLevelSpawn) -> Self {
    let mut by_visual: Vec<Vec<usize>> = vec![Vec::new(); spawn.visuals.len()];

    for (index, object) in spawn.objects.iter().enumerate() {
      if let Some(objects) = by_visual.get_mut(object.visual as usize) {
        objects.push(index);
      }
    }

    Self { spawn, by_visual }
  }

  /// The objects standing as the visual at `visual`, in the spawn's order.
  pub fn list_objects(&self, visual: u32) -> impl Iterator<Item = &RenderSpawnObject> {
    self
      .by_visual
      .get(visual as usize)
      .into_iter()
      .flatten()
      .map(|index| &self.spawn.objects[*index])
  }
}
