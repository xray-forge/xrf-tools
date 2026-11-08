use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::WorldState;

use crate::viewer::recording::{LevelRecordings, RecordingOutcome};

/// Stop the recording, running or ended by itself, and finish its video; the camera heard follows the focus again.
/// Answers what was written.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "stop_recording")]
pub(crate) async fn levels_stop_recording(
  world: State<'_, WorldState>,
  execution: State<'_, ExecutionState>,
  recordings: State<'_, LevelRecordings>,
) -> TauriResult<RecordingOutcome> {
  world.lock().pin_listener(None);

  #[cfg(windows)]
  {
    let recorder = recordings.lock().take().ok_or("No recording runs")?;

    execution
      .run_blocking("Finishing a recording", move || recorder.stop())
      .await?
  }

  #[cfg(not(windows))]
  {
    let _ = (execution, recordings);

    Err(String::from("No recording runs"))
  }
}
