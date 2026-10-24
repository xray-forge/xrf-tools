use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::parts::LevelTexturesPart;
use crate::plugins::levels::state::{LevelState, LevelTextureReference, SelectedLevel};

/// Describe what every texture the open level's surfaces bind resolved to; resolved again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_textures"))]
#[tauri::command(rename = "describe_textures")]
pub async fn levels_describe_textures(
  session_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Vec<LevelTextureReference>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.levels.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing the level's textures", move || {
      let textures: Arc<Vec<LevelTextureReference>> = LevelTexturesPart::get(&current, &assets.search(&current.roots)?);

      TauriResult::Ok(current.answer(textures.to_vec()))
    })
    .await?
}
