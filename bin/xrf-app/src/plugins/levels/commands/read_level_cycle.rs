use std::sync::Arc;

use tauri::State;
use xrf_environment::{WeatherCycle, WeatherCycleId};

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelEnvironment, LevelState, LevelWeatherCycle, SelectedLevel};

/// Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
/// not offer itself.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_level_cycle"))]
#[tauri::command(rename = "read_level_cycle")]
pub async fn levels_read_level_cycle(
  session_id: SessionId,
  cycle: WeatherCycleId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherCycle>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let cycle: LevelWeatherCycle = execution
    .run_blocking("Reading a weather cycle", move || {
      let environment: Arc<LevelEnvironment> = LevelEnvironment::of(&current)?;
      let found: &WeatherCycle = environment.catalog.find_by_id(&cycle).ok_or_else(|| {
        format!(
          "There is no {} '{}'",
          cycle.kind.get_subject().to_lowercase(),
          cycle.name
        )
      })?;

      assets.with_probe(&current.roots, |probe| {
        LevelWeatherCycle::of(found, &environment.get_source(&current, probe))
      })
    })
    .await??;

  Ok(SessionSnapshot {
    session_id,
    value: cycle,
  })
}
