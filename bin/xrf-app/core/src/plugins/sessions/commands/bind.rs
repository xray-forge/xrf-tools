use tauri::State;

use crate::session::{SessionId, SessionScope, SessionScopes};

/// Bind an opening to the tool that asks for it, before its open, so whatever releases the tool's scope closes it; an
/// opening not bound is refused.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "bind"))]
#[tauri::command(rename = "bind")]
pub fn sessions_bind(scopes: State<'_, SessionScopes>, session_id: SessionId, scope: SessionScope) {
  scopes.bind(session_id, scope);
}
