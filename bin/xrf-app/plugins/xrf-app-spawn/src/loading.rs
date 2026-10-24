use std::path::PathBuf;

use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionOpening};
use xrf_app_core::types::TauriResult;
use xrf_spawn::{SpawnFile, XRayByteOrder};

use crate::SpawnSessionDescriptor;
use crate::state::{SpawnFileState, SpawnSession};

/// The two representations accepted by the editor's open commands.
pub enum SpawnInput {
  Packed,
  Unpacked,
}

/// Parse on the application pool, then publish only if this opening still owns its reservation.
pub async fn open_spawn(
  id: SessionId,
  path: PathBuf,
  input: SpawnInput,
  state: &SpawnFileState,
  execution: &ExecutionState,
) -> TauriResult<SpawnSessionDescriptor> {
  let opening: SessionOpening<SpawnSession> = state.files.begin_open(id)?;

  execution
    .run_blocking("Opening spawn", move || {
      let file: SpawnFile = match input {
        SpawnInput::Packed => SpawnFile::read_from_path::<XRayByteOrder, _>(&path),
        SpawnInput::Unpacked => SpawnFile::import_from_path::<XRayByteOrder, _>(&path),
      }
      .map_err(|error| format!("Cannot open spawn '{}': {error}", path.display()))?;

      SpawnFileState::commit_open(opening, path, file)
    })
    .await?
}
