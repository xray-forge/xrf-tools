use xrf_renderer::RenderSpawnModel;

use crate::scene::SceneSurfaceKind;

/// The models a library object compiles to (`SceneObjectCompiler::compile`), one per surface kind asked for and held,
/// named by `SceneSurfaceKind::format_model_name` and taken as the renderer asks for each visual.
#[derive(Default)]
pub(crate) struct SceneObjectModels {
  models: [Option<RenderSpawnModel>; SceneSurfaceKind::ALL.len()],
}

impl SceneObjectModels {
  /// Keeps `model` as the model of `kind`.
  pub(crate) fn insert(&mut self, kind: SceneSurfaceKind, model: RenderSpawnModel) {
    self.models[kind.get_index()] = Some(model);
  }

  /// Takes the model of `kind`; none for a kind it holds no face of, not asked for, or taken.
  pub(crate) fn take(&mut self, kind: SceneSurfaceKind) -> Option<RenderSpawnModel> {
    self.models[kind.get_index()].take()
  }
}
