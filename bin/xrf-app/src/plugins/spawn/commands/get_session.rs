use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;
use crate::plugins::spawn::state::SpawnFileState;

/// Restore a coherent session without cloning any of the large chunks.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub fn spawn_get_session(state: State<'_, SpawnFileState>) -> TauriResult<Option<SpawnSessionDescriptor>> {
  state.get_descriptor()
}
