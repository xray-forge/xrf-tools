use tauri::State;

use crate::session::{SessionScope, SessionScopes};
use crate::window::InvokingWindow;

/// Keep only the openings of the asking window's live tools, closing what any other tool of it left, as a page starts
/// and knows which tools it runs: a page that reloaded could not release what it was leaving. Answers how many were
/// closed.
///
/// Synchronous, as `release_scope` is, so it unbinds in the order the page sent its calls.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "retain_scopes")]
pub fn sessions_retain_scopes(scopes: State<'_, SessionScopes>, window: InvokingWindow, tools: Vec<String>) -> u32 {
  let label: &str = window.get_label();
  let live: Vec<SessionScope> = tools.into_iter().map(|tool| SessionScope::new(label, tool)).collect();
  let released: usize = scopes.retain(label, &live);

  if released > 0 {
    log::info!("Released {released} opening(s) window '{label}' no longer holds");
  }

  released as u32
}
