use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::WorldState;

use crate::viewer::recording::{LevelRecordings, RecordingOutcome};

/// Stop the recording, running or self-ended, and finish its video; answers what was written. The heard camera follows
/// focus again.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "stop_recording")]
pub(crate) async fn levels_stop_recording(
  world: State<'_, WorldState>,
  execution: State<'_, ExecutionState>,
  recordings: State<'_, LevelRecordings>,
) -> TauriResult<RecordingOutcome> {
  let world: WorldState = world.inner().clone();
  let recordings: LevelRecordings = recordings.inner().clone();

  // Taken on the pool: a start holds the slot while its recorder opens.
  execution
    .run_blocking("Finishing a recording", move || recordings.stop(&world))
    .await?
}
