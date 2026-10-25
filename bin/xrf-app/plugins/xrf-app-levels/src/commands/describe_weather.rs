use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::LevelEnvironmentPart;
use crate::weather::LevelWeatherDescription;

/// Describe the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
/// the sun table and the level's own overrides.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_weather")]
pub async fn levels_describe_weather(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherDescription>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Reading the level weather",
    |current, search| {
      LevelEnvironmentPart::get(current, search).map(|environment| environment.describe(current, search))
    },
  )
  .await
}
