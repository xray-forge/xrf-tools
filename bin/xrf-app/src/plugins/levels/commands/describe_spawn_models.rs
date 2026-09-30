use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_vfs::XrayProbe;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::report::report_spawn_models;
use crate::plugins::levels::spawn_visuals::SpawnVisualReader;
use crate::plugins::levels::state::{
  LevelSpawnModelDescription, LevelSpawnModelFailure, LevelSpawnModelsDescription, LevelSpawnVisual, LevelState,
  SelectedLevel,
};

/// Describe the models of a batch of the visuals open_spawn_objects named, reading and packing each once.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_spawn_models"))]
#[tauri::command(rename = "describe_spawn_models")]
pub async fn levels_describe_spawn_models(
  session_id: SessionId,
  names: Vec<String>,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnModelsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: LevelSpawnModelsDescription = execution
    .run_blocking("Describing the level spawned models", move || {
      assets.with_probe(&current.roots, |probe| describe_models(&current, probe, &names))
    })
    .await??;

  Ok(SessionSnapshot { session_id, value })
}

fn describe_models(current: &SelectedLevel, probe: &XrayProbe, names: &[String]) -> LevelSpawnModelsDescription {
  let started: Instant = Instant::now();
  let visuals: SpawnVisualReader = SpawnVisualReader::new(current, probe);
  let mut described: LevelSpawnModelsDescription = LevelSpawnModelsDescription {
    failures: Vec::new(),
    models: Vec::new(),
  };

  for name in names {
    match visuals.get(name) {
      Ok(visual) => described.models.push(describe(name, &visual)),
      Err(reason) => described.failures.push(LevelSpawnModelFailure {
        name: name.clone(),
        reason,
      }),
    }
  }

  report_spawn_models(&current.source, &described, started);

  described
}

fn describe(name: &str, visual: &LevelSpawnVisual) -> LevelSpawnModelDescription {
  LevelSpawnModelDescription {
    description: visual.package.description.clone(),
    name: name.to_owned(),
    rest: visual.rest.as_ref().map(|pose| pose.to_floats()),
    surfaces: visual.surfaces.clone(),
    textures: visual.textures.clone(),
  }
}
