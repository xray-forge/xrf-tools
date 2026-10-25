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
use xrf_renderer::CubeAssembly;

use crate::viewer::cube::CubeCaptureRequest;
use crate::viewer::{LevelViewerReport, ViewerCaptureJob};

/// Capture a cubemap around a viewport's camera face by face and write it to a PNG file as a horizontal cross.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "capture_cube")]
pub async fn levels_capture_cube(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: CubeCaptureRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<CaptureOutcome> {
  let CubeCaptureRequest {
    viewport,
    capture,
    path,
  } = request;

  LevelViewerReport::capturing_cube(&capture, viewport, &path);

  ViewerCaptureJob {
    viewport,
    path,
    job_id,
    progress,
  }
  .run::<CubeAssembly>(capture, &renderer, &directories, &registry, &execution)
  .await
}
