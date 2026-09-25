use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_visual::to_spawn_transform;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_visuals::{get_drawn_visual, get_spawn_visual};
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
) -> TauriResult<SessionSnapshot<LevelSpawnModelsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let started: Instant = Instant::now();
  let value: LevelSpawnModelsDescription = assets.with_probe(&current.roots, |probe| {
    let mut models: Vec<LevelSpawnModelDescription> = Vec::new();
    let mut indices: HashMap<String, Option<u32>> = HashMap::new();
    let mut placements: Vec<LevelSpawnPlacement> = Vec::new();

    let spawn = match get_level_spawn(&current, probe) {
      Ok(spawn) => spawn,
      Err(error) => {
        log::warn!("No spawned models for {}: {error}", current.source.get_label());

        return LevelSpawnModelsDescription { models, placements };
      }
    };

    for object in &spawn.objects {
      let Some(name) = get_drawn_visual(object) else {
        continue;
      };

      let model: Option<u32> = *indices.entry(name.to_owned()).or_insert_with(|| {
        get_spawn_visual(&current, probe, name).map(|visual| {
          models.push(describe(name, &visual));

          (models.len() - 1) as u32
        })
      });

      if let Some(model) = model {
        placements.push(LevelSpawnPlacement {
          model,
          name: object.name.clone(),
          section: object.section.clone(),
          transform: to_spawn_transform(&object.position, &object.direction),
        });
      }
    }

    LevelSpawnModelsDescription { models, placements }
  })?;

  log::info!(
    "Described the spawned models of {}: {} models, {} placed, in {:?}",
    current.source.get_label(),
    value.models.len(),
    value.placements.len(),
    started.elapsed()
  );

  Ok(SessionSnapshot { session_id, value })
}

fn describe(name: &str, visual: &LevelSpawnVisual) -> LevelSpawnModelDescription {
  LevelSpawnModelDescription {
    description: visual.package.description.clone(),
    name: name.to_owned(),
    rest: visual.rest.as_ref().map(|pose| {
      pose
        .transforms
        .iter()
        .flat_map(|it| {
          [
            it.i.x, it.i.y, it.i.z, it.j.x, it.j.y, it.j.z, it.k.x, it.k.y, it.k.z, it.c.x, it.c.y, it.c.z,
          ]
        })
        .collect()
    }),
    surfaces: visual.surfaces.clone(),
    textures: visual.textures.clone(),
  }
}
