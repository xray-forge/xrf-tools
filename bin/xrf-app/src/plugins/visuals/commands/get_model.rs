use tauri::State;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, SelectedVisualDescription, VisualState};

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
