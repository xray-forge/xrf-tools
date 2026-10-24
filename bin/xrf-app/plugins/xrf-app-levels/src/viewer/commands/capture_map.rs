use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_app_render::{CaptureOutcome, Renderer};
use xrf_job::JobProgress;
use xrf_renderer::MapAssembly;

use crate::viewer::map::MapCaptureRequest;
use crate::viewer::{LevelViewerReport, ViewerCaptureJob};

/// Capture a level map in a viewport tile by tile and write it to a PNG file with its coverage in alpha.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "capture_map"))]
#[tauri::command(rename = "capture_map")]
pub async fn levels_capture_map(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: MapCaptureRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<CaptureOutcome> {
  let MapCaptureRequest {
    viewport,
    capture,
    path,
  } = request;

  LevelViewerReport::capturing_map(&capture, viewport, &path);

  ViewerCaptureJob {
    viewport,
    path,
    job_id,
    progress,
  }
  .run::<MapAssembly>(capture, &renderer, &directories, &registry, &execution)
  .await
}
