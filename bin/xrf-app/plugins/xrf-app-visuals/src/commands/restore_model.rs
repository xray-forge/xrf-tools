use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::{SelectedVisual, SelectedVisualDescription, VisualModels};

/// The model `tool` holds in the asking window, the newest it opened, with its exact geometry identity.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "restore_model"))]
#[tauri::command(rename = "restore_model")]
pub async fn visuals_restore_model(
  models: State<'_, VisualModels>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<Arc<SelectedVisualDescription>>> {
  Ok(
    models
      .restore_as(&SessionScope::of(&window, tool), SelectedVisual::describe)
      .into(),
  )
}
