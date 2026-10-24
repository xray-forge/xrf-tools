use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;

/// Close every opening a tool held, in every domain, as the tool is left; answers how many there were.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "release_scope"))]
#[tauri::command(rename = "release_scope")]
pub async fn sessions_release_scope(
  scopes: State<'_, SessionScopes>,
  execution: State<'_, ExecutionState>,
  scope: SessionScope,
) -> TauriResult<u32> {
  let scopes: SessionScopes = SessionScopes::clone(&scopes);

  // On the pool, as the documents it closes, whole levels among them, are dropped as they close.
  execution
    .run_blocking("Releasing a tool's openings", move || {
      let released: usize = scopes.release(&scope);

      log::info!(
        "Released {released} opening(s) of tool '{}' in window '{}'",
        scope.tool,
        scope.window
      );

      released as u32
    })
    .await
}
