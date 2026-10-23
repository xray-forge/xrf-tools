use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};

/// Keep only the openings of a window's live tools, closing what any other tool of it left, as a page starts and knows
/// which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were closed.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "retain_scopes"))]
#[tauri::command(rename = "retain_scopes")]
pub fn sessions_retain_scopes(scopes: State<'_, SessionScopes>, window: String, live: Vec<SessionScope>) -> u32 {
  let released: usize = scopes.retain(&window, &live);

  if released > 0 {
    log::info!("Released {released} opening(s) window '{window}' no longer holds");
  }

  released as u32
}
