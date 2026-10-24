use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;

/// Close every opening a tool held, in every domain, as the tool is left; answers how many there were.
// Async, so the documents it closes are dropped off the main thread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "release_scope"))]
#[tauri::command(rename = "release_scope")]
pub async fn sessions_release_scope(scopes: State<'_, SessionScopes>, scope: SessionScope) -> TauriResult<u32> {
  let released: usize = scopes.release(&scope);

  log::info!(
    "Released {released} opening(s) of tool '{}' in window '{}'",
    scope.tool,
    scope.window
  );

  Ok(released as u32)
}
