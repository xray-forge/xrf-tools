use std::path::PathBuf;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;
use crate::plugins::spawn::loading::{SpawnInput, open_spawn};
use crate::plugins::spawn::state::SpawnFileState;

/// Open a packed spawn and return its identity, path, and header together.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_file"))]
#[tauri::command(rename = "open_file")]
pub async fn spawn_open_file(
  session_id: SessionId,
  path: PathBuf,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnSessionDescriptor> {
  open_spawn(session_id, path, SpawnInput::Packed, &state, &execution).await
}
