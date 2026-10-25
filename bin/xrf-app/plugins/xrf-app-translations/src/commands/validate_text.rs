use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionId;
use xrf_translation::find_unwritable_character;
use xrf_utils::error_to_string;

use crate::TranslationProjects;

/// Report the first character a language cannot hold, or nothing when the value is writable.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "validate_text")]
pub async fn translations_validate_text(
  session_id: SessionId,
  language: &str,
  text: &str,
  projects: State<'_, TranslationProjects>,
) -> TauriResult<Option<String>> {
  let project = projects.require(session_id)?;

  find_unwritable_character(&project, language, text).map_err(error_to_string)
}
