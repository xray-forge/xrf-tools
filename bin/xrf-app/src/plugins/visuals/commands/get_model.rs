use std::sync::MutexGuard;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, SelectedVisualDescription, VisualState};

/// What the viewer had selected, or null when nothing is open.
///
/// This is the rehydration probe: a reloaded frontend asks what is selected and then asks for that
/// source's geometry, so the selection survives a reload without the frontend storing anything.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_model"))]
#[tauri::command(rename = "get_model")]
pub async fn visuals_get_model(state: State<'_, VisualState>) -> TauriResult<Option<SelectedVisualDescription>> {
  let selected: MutexGuard<Option<SelectedVisual>> = state
    .selected
    .lock()
    .map_err(|error| format!("Failed to read visual - selection state is unavailable: {error}"))?;

  match selected.as_ref() {
    Some(current) => {
      log::info!("Reporting selected visual: {}", current.source.label());

      Ok(Some(SelectedVisualDescription {
        source: current.source.clone(),
        description: current.package.description.clone(),
        roots: current.roots.clone(),
        dependencies: current.dependencies.clone(),
        textures: current.textures.clone(),
      }))
    }
    None => {
      log::info!("Reporting no selected visual");

      Ok(None)
    }
  }
}
