use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::VisualModels;

/// Every motion the open visual can play, by name, read on the pool.
///
/// Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
/// references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
/// visual animates at all, from its references, so nothing needs this until something is about to play one.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_motions")]
pub async fn visuals_list_motions(
  session_id: SessionId,
  models: State<'_, VisualModels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Vec<String>>> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  models
    .read_on_pool(&execution, session_id, "Listing a visual's motions", move |current| {
      let Some(motions) = current.motions.as_ref() else {
        // No bind pose means nothing to pose, so the names would be unplayable even where they exist.
        return Ok(Vec::new());
      };
      let names: Vec<String> = motions.list_names(
        &assets.search(&current.description.roots)?,
        &current.description.dependencies,
      );

      log::info!(
        "Listed {} motions for: {}",
        names.len(),
        current.description.source.label()
      );

      Ok(names)
    })
    .await
}
