use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_vfs::XrayProbe;
use xrf_visual::{HemiEstimator, VisualSphere};

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::hemi::{estimate_spawn_hemi, get_level_hemi};
use crate::plugins::levels::report::report_spawn_models;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_visuals::SpawnVisualReader;
use crate::plugins::levels::state::{
  LevelSpawn, LevelSpawnModelDescription, LevelSpawnModelFailure, LevelSpawnModelsDescription, LevelSpawnObjectHemi,
  LevelSpawnVisual, LevelState, SelectedLevel,
};

/// Describe the models of a batch of the visuals open_spawn_objects named, reading and packing each once, and how the
/// level lights every object standing as one of them.
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
  // The level's lighting is built beside the first batch's visuals rather than after them; later batches find it.
  let ((mut described, read), estimator) =
    rayon::join(|| read_models(current, probe, names), || get_level_hemi(current, probe));

  if let Ok(estimator) = estimator {
    described.hemi = describe_hemi(current, probe, &estimator, &read);
  }

  // Told which visuals are described, the lighting is let go after the last.
  current.spawn_lighting.note_described(names);

  report_spawn_models(&current.source, &described, started);

  described
}

/// Each visual of a batch described, and each read beside its name.
fn read_models<'names>(
  current: &SelectedLevel,
  probe: &XrayProbe,
  names: &'names [String],
) -> (LevelSpawnModelsDescription, Vec<(&'names str, Arc<LevelSpawnVisual>)>) {
  let visuals: SpawnVisualReader = SpawnVisualReader::new(current, probe);
  let mut described: LevelSpawnModelsDescription = LevelSpawnModelsDescription {
    failures: Vec::new(),
    hemi: Vec::new(),
    models: Vec::new(),
  };
  let mut read: Vec<(&str, Arc<LevelSpawnVisual>)> = Vec::new();

  for name in names {
    match visuals.get(name) {
      Ok(visual) => {
        described.models.push(describe(name, &visual));
        read.push((name, visual));
      }
      Err(reason) => described.failures.push(LevelSpawnModelFailure {
        name: name.clone(),
        reason,
      }),
    }
  }

  (described, read)
}

/// How the level lights each object standing as one of the visuals read, none where its spawn cannot be read.
fn describe_hemi(
  current: &SelectedLevel,
  probe: &XrayProbe,
  estimator: &HemiEstimator,
  read: &[(&str, Arc<LevelSpawnVisual>)],
) -> Vec<LevelSpawnObjectHemi> {
  let Ok(spawn) = get_level_spawn(current, probe) else {
    return Vec::new();
  };
  let spawn: &LevelSpawn = &spawn;
  let spheres: HashMap<&str, &VisualSphere> = read
    .iter()
    .map(|(name, visual)| (*name, &visual.package.description.declared_bounds.bounding_sphere))
    .collect();

  estimate_spawn_hemi(estimator, spawn, &spheres)
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
