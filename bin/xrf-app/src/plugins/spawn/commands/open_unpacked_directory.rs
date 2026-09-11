use std::path::PathBuf;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;
use crate::plugins::spawn::loading::{SpawnInput, open_spawn};
use crate::plugins::spawn::state::SpawnFileState;

/// Open a unpacked spawn and return its identity, path, and header together.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_unpacked_directory"))]
#[tauri::command(rename = "open_unpacked_directory")]
pub async fn spawn_open_unpacked_directory(
  path: PathBuf,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnSessionDescriptor> {
  open_spawn(path, SpawnInput::Unpacked, &state, &execution).await
}
