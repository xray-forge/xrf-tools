use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_vfs::XrayProbe;
use xrf_visual::VisualTransform;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::report::{report_missing_spawn_models, report_spawn_models};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_visuals::{SpawnVisualReader, get_drawn_visual};
use crate::plugins::levels::state::{
  LevelSpawnModelDescription, LevelSpawnModelsDescription, LevelSpawnPlacement, LevelSpawnVisual, LevelState,
  SelectedLevel,
};

/// Describe the models the open level's spawned objects are drawn as, and where each object stands.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_spawn_models"))]
#[tauri::command(rename = "open_spawn_models")]
pub async fn levels_open_spawn_models(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnModelsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: LevelSpawnModelsDescription = execution
    .run_blocking("Describing the level spawned models", move || {
      assets.with_probe(&current.roots, |probe| describe_models(&current, probe))
    })
    .await??;

  Ok(SessionSnapshot { session_id, value })
}

fn describe_models(current: &SelectedLevel, probe: &XrayProbe) -> LevelSpawnModelsDescription {
  let started: Instant = Instant::now();
  let mut models: Vec<LevelSpawnModelDescription> = Vec::new();
  let mut placements: Vec<LevelSpawnPlacement> = Vec::new();

  let spawn = match get_level_spawn(current, probe) {
    Ok(spawn) => spawn,
    Err(error) => {
      report_missing_spawn_models(&current.source, &error);

      return LevelSpawnModelsDescription { models, placements };
    }
  };

  let visuals: SpawnVisualReader = SpawnVisualReader::new(current, probe);
  let mut indices: HashMap<&str, Option<u32>> = HashMap::new();

  for object in &spawn.objects {
    let Some(name) = get_drawn_visual(object) else {
      continue;
    };

    let model: Option<u32> = *indices.entry(name).or_insert_with(|| {
      visuals.get(name).map(|visual| {
        models.push(describe(name, &visual));

        (models.len() - 1) as u32
      })
    });

    if let Some(model) = model {
      placements.push(LevelSpawnPlacement {
        model,
        name: object.name.clone(),
        section: object.section.clone(),
        transform: VisualTransform::of_spawn(&object.position, &object.direction),
      });
    }
  }

  let described: LevelSpawnModelsDescription = LevelSpawnModelsDescription { models, placements };

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
