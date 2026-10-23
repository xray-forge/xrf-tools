use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};

/// Close every opening a tool held, in every domain, as the tool is left; answers how many there were.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "release_scope"))]
#[tauri::command(rename = "release_scope")]
pub fn sessions_release_scope(scopes: State<'_, SessionScopes>, scope: SessionScope) -> u32 {
  let released: usize = scopes.release(&scope);

  log::info!(
    "Released {released} opening(s) of tool '{}' in window '{}'",
    scope.tool,
    scope.window
  );

  released as u32
}
