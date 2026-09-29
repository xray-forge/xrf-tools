use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelEnvironment, LevelState, LevelWeatherDescription, SelectedLevel};

/// Read the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
/// the sun table and the level's own overrides.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_level_weather"))]
#[tauri::command(rename = "read_level_weather")]
pub async fn levels_read_level_weather(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let description: LevelWeatherDescription = execution
    .run_blocking("Reading the level weather", move || {
      // The configs mount a tree of their own, so they are read before the probe the level's own file is read in.
      let environment: Arc<LevelEnvironment> = LevelEnvironment::of(&current)?;

      assets.with_probe(&current.roots, |probe| environment.describe(&current, probe))
    })
    .await??;

  Ok(SessionSnapshot {
    session_id,
    value: description,
  })
}
