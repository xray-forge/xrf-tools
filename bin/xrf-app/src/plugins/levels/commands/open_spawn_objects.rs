use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_vfs::XrayProbe;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::report::{report_missing_spawn_objects, report_spawn_objects};
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::spawn_objects::describe_spawn_objects;
use crate::plugins::levels::state::{LevelSpawn, LevelSpawnObjectsDescription, LevelState, SelectedLevel};

/// Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
/// error where the spawn cannot be read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_spawn_objects"))]
#[tauri::command(rename = "open_spawn_objects")]
pub async fn levels_open_spawn_objects(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnObjectsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: LevelSpawnObjectsDescription = execution
    .run_blocking("Describing the level spawned objects", move || {
      assets.with_probe(&current.roots, |probe| describe_objects(&current, probe))
    })
    .await???;

  Ok(SessionSnapshot { session_id, value })
}

fn describe_objects(current: &SelectedLevel, probe: &XrayProbe) -> TauriResult<LevelSpawnObjectsDescription> {
  let started: Instant = Instant::now();
  let spawn: Arc<LevelSpawn> =
    get_level_spawn(current, probe).inspect_err(|error| report_missing_spawn_objects(&current.source, error))?;
  let described: LevelSpawnObjectsDescription = describe_spawn_objects(&spawn);

  // Every visual named is one a batch will describe, and the lighting is held until the last of them is.
  current.spawn_lighting.expect(&described.visuals)?;
  report_spawn_objects(&current.source, &described, started);

  Ok(described)
}
