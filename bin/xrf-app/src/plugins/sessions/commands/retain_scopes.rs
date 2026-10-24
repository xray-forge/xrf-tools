use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;

/// Keep only the openings of a window's live tools, closing what any other tool of it left, as a page starts and knows
/// which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were closed.
// Async, so the documents it closes are dropped off the main thread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "retain_scopes"))]
#[tauri::command(rename = "retain_scopes")]
pub async fn sessions_retain_scopes(
  scopes: State<'_, SessionScopes>,
  window: String,
  live: Vec<SessionScope>,
) -> TauriResult<u32> {
  let released: usize = scopes.retain(&window, &live);

  if released > 0 {
    log::info!("Released {released} opening(s) window '{window}' no longer holds");
  }

  Ok(released as u32)
}
