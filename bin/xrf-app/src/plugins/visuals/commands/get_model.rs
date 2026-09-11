use tauri::State;

use crate::core::session::DocumentRestore;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;
use crate::plugins::visuals::state::{SelectedVisual, SelectedVisualDescription};

/// Restore the committed model descriptor and its exact geometry identity.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_model"))]
#[tauri::command(rename = "get_model")]
pub async fn visuals_get_model(
  state: State<'_, VisualState>,
) -> TauriResult<DocumentRestore<SelectedVisualDescription>> {
  Ok(DocumentRestore::from(
    state.selected.get()?.map(|opened| opened.map(SelectedVisual::describe)),
  ))
}
