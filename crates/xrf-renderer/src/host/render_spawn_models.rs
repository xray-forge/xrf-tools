use crate::host::render_spawn_lighting::RenderSpawnLighting;
use crate::host::render_spawn_model::RenderSpawnModel;

/// A batch of spawned visuals read, and how the level lights each object standing as one of them.
#[derive(Debug, Default)]
pub struct RenderSpawnModels {
  pub models: Vec<RenderSpawnModel>,
  /// How the level lights each object standing as one of them; an object it was not estimated for has none.
  pub lighting: Vec<RenderSpawnLighting>,
}
