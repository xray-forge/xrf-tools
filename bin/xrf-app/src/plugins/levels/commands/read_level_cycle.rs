use std::sync::Arc;

use tauri::State;
use xrf_environment::{WeatherCycle, WeatherCycleKind};

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelEnvironment, LevelState, LevelWeatherCycle, SelectedLevel};
use crate::plugins::levels::weather::get_level_environment;

/// Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
/// not offer itself.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_level_cycle"))]
#[tauri::command(rename = "read_level_cycle")]
pub async fn levels_read_level_cycle(
  session_id: SessionId,
  kind: WeatherCycleKind,
  name: String,
  state: State<'_, LevelState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelWeatherCycle>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let cycle: LevelWeatherCycle = execution
    .run_blocking("Reading a weather cycle", move || {
      let environment: Arc<LevelEnvironment> = get_level_environment(&current)?;
      let found: Option<&WeatherCycle> = match kind {
        WeatherCycleKind::Cycle => environment.catalog.find_cycle(&name),
        WeatherCycleKind::Effect => environment.catalog.find_effect(&name),
      };

      found
        .map(|cycle| LevelWeatherCycle::of(cycle, &environment.catalog, current.engine))
        .ok_or_else(|| format!("There is no weather cycle '{name}'"))
    })
    .await??;

  Ok(SessionSnapshot {
    session_id,
    value: cycle,
  })
}
