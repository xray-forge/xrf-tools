use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::parts::{LevelTextureReference, LevelTexturesPart};

/// Describe what every texture the open level's surfaces bind resolved to; resolved again where a commit let it go.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_textures")]
pub async fn levels_describe_textures(
  session_id: SessionId,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<Vec<LevelTextureReference>>>> {
  SelectedLevel::answer(
    &levels,
    &assets,
    &execution,
    session_id,
    "Describing the level's textures",
    |current, search| Ok(LevelTexturesPart::get(current, search)),
  )
  .await
}
