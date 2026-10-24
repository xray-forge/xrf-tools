use tauri::State;
use xrf_app_core::session::{SessionRestore, SessionScope, SessionScopes};
use xrf_app_core::types::TauriResult;

use crate::selected_visual::SelectedVisual;
use crate::selected_visual_description::SelectedVisualDescription;
use crate::state::VisualState;

/// Restore the model a tool holds, the newest it opened, with its exact geometry identity.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_model"))]
#[tauri::command(rename = "get_model")]
pub async fn visuals_get_model(
  state: State<'_, VisualState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<SelectedVisualDescription>> {
  Ok(SessionRestore::from(
    state
      .models
      .restore(&scopes.list(&scope))?
      .map(|opened| opened.map(SelectedVisual::describe)),
  ))
}
