use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_vfs::XraySearch;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::parts::LevelSpawnPart;
use crate::plugins::levels::report::{report_missing_spawn_objects, report_spawn_objects};
use crate::plugins::levels::spawn_objects::describe_spawn_objects;
use crate::plugins::levels::state::{LevelSpawn, LevelSpawnObjectsDescription, LevelState, SelectedLevel};

/// Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
/// error where the spawn cannot be read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_spawn_objects"))]
#[tauri::command(rename = "describe_spawn_objects")]
pub async fn levels_describe_spawn_objects(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnObjectsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: SessionSnapshot<LevelSpawnObjectsDescription> = execution
    .run_blocking("Describing the level spawned objects", move || {
      describe_objects(&current, &assets.search(&current.roots)?).map(|described| current.answer(described))
    })
    .await??;

  Ok(value)
}

fn describe_objects(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
) -> TauriResult<LevelSpawnObjectsDescription> {
  let started: Instant = Instant::now();
  let spawn: Arc<LevelSpawn> =
    LevelSpawnPart::get(current, search).inspect_err(|error| report_missing_spawn_objects(&current.source, error))?;
  let described: LevelSpawnObjectsDescription = describe_spawn_objects(&spawn);

  report_spawn_objects(&current.source, &described, started);

  Ok(described)
}
