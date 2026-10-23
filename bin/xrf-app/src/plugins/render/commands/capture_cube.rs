use std::sync::Arc;
use std::sync::mpsc::Receiver;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_renderer::{CaptureEvent, CaptureShot, CubeAssembly};

use crate::core::app_directories::AppDirectories;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::render::capture_outcome::CaptureOutcome;
use crate::plugins::render::capture_receiver::{CaptureNames, receive_shots, write_capture};
use crate::plugins::render::capture_request::CubeCaptureRequest;
use crate::plugins::render::renderer::Renderer;

/// Capture a cubemap around a viewport's camera face by face and write it to a PNG file as a horizontal cross.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "capture_cube"))]
#[tauri::command(rename = "capture_cube")]
pub async fn render_capture_cube(
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

  LevelViewerFolder::require_screenshot(&directories, &path)?;

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, JobKind::LevelsCaptureCube)
      .with_request(&capture)
      .with_resources(vec![JobResource::file(&path)])
      .with_progress(progress),
  )?;
  let events: Receiver<CaptureEvent> = renderer.capture_cube(viewport, capture);

  log::info!(
    "Capturing a cubemap of {} pixel faces in viewport {} to '{}'",
    capture.size,
    viewport.0,
    path.display()
  );

  run_job(
    &execution,
    "Cubemap capture",
    registration,
    move || {
      // Made at the first face, once the renderer has taken the plan, so a capture it refuses allocates nothing.
      let mut assembly: Option<CubeAssembly> = None;
      let names: CaptureNames = CaptureNames {
        capture: "cubemap",
        shots: "faces",
      };

      let is_finished: bool = receive_shots((&job, &names), &events, |shot, drawn| match shot {
        CaptureShot::Face(face) => assembly
          .get_or_insert_with(|| CubeAssembly::new(capture.size))
          .place(&face, drawn)
          .map_err(|error| error.to_string()),
        CaptureShot::Tile(_) => Err("A cubemap came back with a level map's tile".to_string()),
      })?;

      if !is_finished {
        return Ok(CaptureOutcome::cancelled());
      }

      let assembly: CubeAssembly = assembly.ok_or_else(|| "The renderer drew no face of the cubemap".to_string())?;

      write_capture(&names, &assembly.finish(), &path)
    },
    |outcome: &CaptureOutcome| outcome.outcome,
  )
  .await
}
