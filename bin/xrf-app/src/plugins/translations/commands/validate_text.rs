use tauri::State;
use xrf_translation::find_unwritable_character;
use xrf_utils::error_to_string;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

/// Report the first character a language cannot hold, or nothing when the value is writable.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "validate_text"))]
#[tauri::command(rename = "validate_text")]
pub async fn translations_validate_text(
  session_id: SessionId,
  language: &str,
  text: &str,
  state: State<'_, TranslationProjectState>,
) -> TauriResult<Option<String>> {
  let project = state.session.require(session_id)?;

  find_unwritable_character(&project, language, text).map_err(error_to_string)
}
