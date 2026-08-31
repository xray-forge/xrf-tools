use tauri::State;
use xrf_translation::TranslationProjectDescriptor;

use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn translations_get_project(
  state: State<'_, TranslationProjectState>,
) -> TauriResult<Option<TranslationProjectDescriptor>> {
  log::info!("Getting translations project");

  state.get_project()
}
