use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::Renderer;
use xrf_renderer::BurstEvent;

use crate::viewer::LevelViewerFolder;
use crate::viewer::burst::{BurstCaptureRequest, BurstOutcome, BurstWriter};

/// Draw a burst of consecutive frames in a viewport and write each, with its G-buffer's marks, into a new folder of
/// the level viewer's bursts named as asked; answers once the last is written.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_burst")]
pub async fn levels_save_burst(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: BurstCaptureRequest,
) -> TauriResult<BurstOutcome> {
  let BurstCaptureRequest { viewport, burst, name } = request;
  let bursts: PathBuf = LevelViewerFolder::Bursts.get_path(&directories)?;
  let renderer: Renderer = renderer.inner().clone();

  execution
    .run_blocking("Saving a burst", move || -> TauriResult<BurstOutcome> {
      let folder: PathBuf = LevelViewerFolder::make(LevelViewerFolder::find_new_burst(&bursts, &name)?)?;
      let events: Receiver<BurstEvent> = renderer.capture_burst(viewport, burst);

      BurstWriter::new(folder, &burst).receive(&events)
    })
    .await?
}
