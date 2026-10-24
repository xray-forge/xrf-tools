use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_environment::{WeatherCycle, WeatherCycleId};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::{LevelEnvironment, LevelEnvironmentPart};
use crate::weather::LevelWeatherCycle;

/// Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
/// not offer itself.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_level_cycle"))]
#[tauri::command(rename = "read_level_cycle")]
pub async fn levels_read_level_cycle(
  session_id: SessionId,
  cycle: WeatherCycleId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherCycle>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Reading a weather cycle",
    move |current, search| {
      let environment: Arc<LevelEnvironment> = LevelEnvironmentPart::get(current, search)?;
      let found: &WeatherCycle = environment.catalog.find_by_id(&cycle).ok_or_else(|| {
        format!(
          "There is no {} '{}'",
          cycle.kind.get_subject().to_lowercase(),
          cycle.name
        )
      })?;

      Ok(LevelWeatherCycle::of(found, &environment.catalog))
    },
  )
  .await
}
