use std::path::PathBuf;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::SpawnSessionDescriptor;
use crate::loading::{SpawnInput, open_spawn};
use crate::state::SpawnFileState;

/// Open a unpacked spawn and return its identity, path, and header together.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_unpacked_directory"))]
#[tauri::command(rename = "open_unpacked_directory")]
pub async fn spawn_open_unpacked_directory(
  session_id: SessionId,
  path: PathBuf,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnSessionDescriptor> {
  open_spawn(session_id, path, SpawnInput::Unpacked, &state, &execution).await
}
