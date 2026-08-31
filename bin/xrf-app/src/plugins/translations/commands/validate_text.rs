use tauri::State;
use xrf_translation::find_unwritable_character;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

/// Report the first character a language cannot hold, or nothing when the value is writable.
///
/// Checked here rather than in the interface because the answer depends on code page tables the
/// browser has no encoder for, and on what each language's own files declared. Called when a cell is
/// committed, so a mistake is reported where it was made instead of at the end of a batch save.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "validate_text"))]
#[tauri::command(rename = "validate_text")]
pub async fn translations_validate_text(
  language: &str,
  text: &str,
  state: State<'_, TranslationProjectState>,
) -> TauriResult<Option<String>> {
  state.with_project(|descriptor| find_unwritable_character(descriptor, language, text).map_err(error_to_string))
}
