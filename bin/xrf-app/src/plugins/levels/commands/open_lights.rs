use std::sync::Arc;

use tauri::State;
use xrf_ltx::Ltx;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::configs::get_level_sections;
use crate::plugins::levels::lights::{PackedLevelLights, pack_lights};
use crate::plugins::levels::report::report_missing_sections;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::state::{LevelLightsDescription, LevelState, SelectedLevel};

/// Collect the open level's lights: the lamps the game spawns on it, and its own.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_lights"))]
#[tauri::command(rename = "open_lights")]
pub async fn levels_open_lights(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<LevelLightsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let packed: PackedLevelLights = execution
    .run_blocking("Collecting the level lights", move || {
      // The sections are read between two probes, as the configs mount a tree of their own.
      let sections: Option<Arc<Ltx>> = assets
        .with_probe(&current.roots, |probe| get_level_spawn(&current, probe))?
        .and_then(|spawn| get_level_sections(&current, &spawn))
        .inspect_err(report_missing_sections)
        .ok();

      assets.with_probe(&current.roots, |probe| {
        pack_lights(&current, probe, sections.as_deref())
      })
    })
    .await??;

  Ok(SessionSnapshot {
    session_id,
    value: LevelLightsDescription {
      lights: packed.lights,
      projectors: packed.projectors,
    },
  })
}
