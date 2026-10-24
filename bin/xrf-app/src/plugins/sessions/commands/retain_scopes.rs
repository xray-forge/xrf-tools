use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;

/// Keep only the openings of a window's live tools, closing what any other tool of it left, as a page starts and knows
/// which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were closed.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "retain_scopes"))]
#[tauri::command(rename = "retain_scopes")]
pub async fn sessions_retain_scopes(
  scopes: State<'_, SessionScopes>,
  execution: State<'_, ExecutionState>,
  window: String,
  live: Vec<SessionScope>,
) -> TauriResult<u32> {
  let scopes: SessionScopes = SessionScopes::clone(&scopes);

  // On the pool, as the documents it closes are dropped as they close.
  execution
    .run_blocking("Releasing what a window's previous page held", move || {
      let released: usize = scopes.retain(&window, &live);

      if released > 0 {
        log::info!("Released {released} opening(s) window '{window}' no longer holds");
      }

      released as u32
    })
    .await
}
