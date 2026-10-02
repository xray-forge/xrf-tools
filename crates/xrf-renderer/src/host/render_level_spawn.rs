use crate::host::render_spawn_object::RenderSpawnObject;

/// The objects of a level's spawn the viewer draws, and the visuals they stand as, each named once.
#[derive(Clone, Debug, Default)]
pub struct RenderLevelSpawn {
  pub visuals: Vec<String>,
  pub objects: Vec<RenderSpawnObject>,
}
