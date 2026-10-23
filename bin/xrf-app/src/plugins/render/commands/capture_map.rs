use std::sync::Arc;
use std::sync::mpsc::Receiver;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_renderer::{CaptureEvent, CaptureShot, MapAssembly};

use crate::core::app_directories::AppDirectories;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::render::capture_outcome::CaptureOutcome;
use crate::plugins::render::capture_receiver::{CaptureNames, receive_shots, write_capture};
use crate::plugins::render::capture_request::MapCaptureRequest;
use crate::plugins::render::renderer::Renderer;

/// Capture a level map in a viewport tile by tile and write it to a PNG file with its coverage in alpha.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "capture_map"))]
#[tauri::command(rename = "capture_map")]
pub async fn render_capture_map(
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

  LevelViewerFolder::require_screenshot(&directories, &path)?;

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, JobKind::LevelsCaptureMap)
      .with_request(&capture)
      .with_resources(vec![JobResource::file(&path)])
      .with_progress(progress),
  )?;
  let events: Receiver<CaptureEvent> = renderer.capture_map(viewport, capture);

  log::info!(
    "Capturing a {} by {} level map in viewport {} to '{}'",
    capture.width,
    capture.height,
    viewport.0,
    path.display()
  );

  run_job(
    &execution,
    "Level map capture",
    registration,
    move || {
      // Made at the first tile, once the renderer has taken the plan, so a capture it refuses allocates nothing.
      let mut assembly: Option<MapAssembly> = None;
      let names: CaptureNames = CaptureNames {
        capture: "level map",
        shots: "tiles",
      };

      let is_finished: bool = receive_shots((&job, &names), &events, |shot, drawn| match shot {
        CaptureShot::Tile(tile) => assembly
          .get_or_insert_with(|| MapAssembly::new(capture.width, capture.height))
          .place(&tile, drawn)
          .map_err(|error| error.to_string()),
        CaptureShot::Face(_) => Err("A level map came back with a cubemap's face".to_string()),
      })?;

      if !is_finished {
        return Ok(CaptureOutcome::cancelled());
      }

      let assembly: MapAssembly = assembly.ok_or_else(|| "The renderer drew no tile of the level map".to_string())?;

      write_capture(&names, &assembly.finish(), &path)
    },
    |outcome: &CaptureOutcome| outcome.outcome,
  )
  .await
}
