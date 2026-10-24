use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::{AlifeObject, AlifeObjectAbstract};
use xrf_vfs::XraySearch;

use crate::parts::LevelSpawnPart;
use crate::state::{LevelSpawnObjectDetails, LevelState, SelectedLevel};

/// Describe one of the open level's spawned objects, by its place among them, as describe_spawn_objects numbered it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_spawn_object"))]
#[tauri::command(rename = "describe_spawn_object")]
pub async fn levels_describe_spawn_object(
  session_id: SessionId,
  index: u32,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelSpawnObjectDetails>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let value: SessionSnapshot<LevelSpawnObjectDetails> = execution
    .run_blocking("Describing a level spawned object", move || {
      describe_object(&current, &assets.search(&current.roots)?, index).map(|details| current.answer(details))
    })
    .await??;

  Ok(value)
}

fn describe_object(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
  index: u32,
) -> TauriResult<LevelSpawnObjectDetails> {
  let spawn = LevelSpawnPart::get(current, search)?;
  let object: &AlifeObject = spawn
    .objects
    .get(index as usize)
    .ok_or_else(|| format!("The level spawns no object {index}"))?;
  let base: Option<&AlifeObjectAbstract> = object.inherited.get_abstract();

  Ok(LevelSpawnObjectDetails {
    custom_data: base.map(|it| it.custom_data.clone()).unwrap_or_default(),
    game_vertex_id: base.map_or(u16::MAX, |it| it.game_vertex_id),
    id: object.id,
    level_vertex_id: base.map_or(u32::MAX, |it| it.level_vertex_id),
  })
}
