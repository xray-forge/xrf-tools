use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_app_render::{CaptureOutcome, Renderer};
use xrf_job::JobProgress;
use xrf_renderer::{CaptureEvent, RenderViewportId};

use crate::viewer::{LevelViewerFolder, ViewerAssembly};

/// A viewport's capture run as a job into a PNG of the screenshots folder: the one run a level map and a cubemap share.
pub struct ViewerCaptureJob {
  pub viewport: RenderViewportId,
  pub path: PathBuf,
  pub job_id: Uuid,
  pub progress: Channel<JobProgress>,
}

impl ViewerCaptureJob {
  /// Captures `plan` and writes it, the renderer asked only once the job is admitted, so a refused job draws nothing.
  ///
  /// # Errors
  ///
  /// Returns an error where the path is not a screenshot's, the job is refused, or the capture fails or is not written.
  pub async fn run<A: ViewerAssembly>(
    self,
    plan: A::Plan,
    renderer: &Renderer,
    directories: &AppDirectories,
    registry: &Arc<JobRegistry>,
    execution: &ExecutionState,
  ) -> TauriResult<CaptureOutcome> {
    let Self {
      viewport,
      path,
      job_id,
      progress,
    } = self;

    LevelViewerFolder::require_screenshot(directories, &path)?;

    let start: JobStart = JobStart::new(job_id, A::KIND)
      .with_request(&plan)
      .with_resources(vec![JobResource::file(&path)])
      .with_progress(progress);
    let renderer: Renderer = renderer.clone();

    registry
      .run(
        execution,
        start,
        move |job| {
          let events: Receiver<CaptureEvent> = A::capture(&renderer, viewport, plan);
          // Made at the first shot, once the renderer has taken the plan, so a capture it refuses allocates nothing.
          let mut assembly: Option<A> = None;
          let is_finished: bool = A::NAMES.receive_shots(&job, &events, |shot, drawn| {
            assembly.get_or_insert_with(|| A::of_plan(plan)).place_shot(shot, drawn)
          })?;

          if !is_finished {
            return Ok(CaptureOutcome::cancelled());
          }

          let assembly: A =
            assembly.ok_or_else(|| format!("The renderer drew no {} of the {}", A::NAMES.shots, A::NAMES.capture))?;

          A::NAMES.write(&assembly.into_picture(), &path)
        },
        |outcome: &CaptureOutcome| outcome.outcome,
      )
      .await
  }
}
