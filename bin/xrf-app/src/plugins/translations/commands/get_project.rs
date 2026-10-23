use tauri::State;
use xrf_translation::TranslationProjectDescriptor;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

/// The project a tool holds, the newest it opened: for a page attaching to it, or reading it again after a commit.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn translations_get_project(
  state: State<'_, TranslationProjectState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<TranslationProjectDescriptor>> {
  Ok(SessionRestore::from(state.projects.restore(&scopes.list(&scope))?))
}
