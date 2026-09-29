use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_reference;

/// Resolve texture references as the open level resolves its own, for a viewer drawing one the level does not name.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "resolve_level_textures"))]
#[tauri::command(rename = "resolve_level_textures")]
pub async fn levels_resolve_level_textures(
  session_id: SessionId,
  references: Vec<String>,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Vec<LevelTextureReference>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let resolved: Vec<LevelTextureReference> = execution
    .run_blocking("Resolving level textures", move || {
      assets.with_probe(&current.roots, |probe| {
        let scope = current.source.get_texture_scope();

        references
          .into_iter()
          .map(|reference| LevelTextureReference {
            logical_path: resolve_reference(probe, &scope, &reference),
            reference,
          })
          .collect()
      })
    })
    .await??;

  Ok(SessionSnapshot {
    session_id,
    value: resolved,
  })
}
