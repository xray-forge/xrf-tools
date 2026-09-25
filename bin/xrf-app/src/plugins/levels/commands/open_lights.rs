use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::lights::{PackedLevelLights, pack_lights};
use crate::plugins::levels::state::{LevelLightsDescription, LevelState, SelectedLevel};

/// Collect the open level's lights: the lamps the game spawns on it, and its own.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_lights"))]
#[tauri::command(rename = "open_lights")]
pub async fn levels_open_lights(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<LevelLightsDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let directory: Option<String> = current.source.get_logical_directory();
  let packed: PackedLevelLights = assets.with_probe(&current.roots, |probe| {
    pack_lights(&current, probe, directory.as_deref())
  })?;

  Ok(SessionSnapshot {
    session_id,
    value: LevelLightsDescription {
      lights: packed.lights,
      projectors: packed.projectors,
    },
  })
}
