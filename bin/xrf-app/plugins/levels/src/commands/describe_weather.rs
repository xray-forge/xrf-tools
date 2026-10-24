use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_vfs::XraySearch;

use crate::parts::LevelEnvironmentPart;
use crate::state::{LevelState, LevelWeatherDescription, SelectedLevel};

/// Read the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
/// the sun table and the level's own overrides.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_weather"))]
#[tauri::command(rename = "describe_weather")]
pub async fn levels_describe_weather(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let description: SessionSnapshot<LevelWeatherDescription> = execution
    .run_blocking("Reading the level weather", move || {
      let search: XraySearch = assets.search(&current.roots)?;

      LevelEnvironmentPart::get(&current, &search)
        .map(|environment| current.answer(environment.describe(&current, &search)))
    })
    .await??;

  Ok(description)
}
