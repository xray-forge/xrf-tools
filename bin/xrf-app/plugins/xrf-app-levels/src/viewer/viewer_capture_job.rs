use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use image::RgbaImage;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobKind, JobRegistry, JobResource, JobStart};
use xrf_app_render::{CaptureNames, CaptureOutcome, Renderer};
use xrf_job::JobProgress;
use xrf_renderer::{
  CaptureEvent, CaptureShot, CubeAssembly, MapAssembly, RenderCapture, RenderCubeCapture, RenderMapCapture,
  RenderViewportId,
};

use crate::ipc::levels::{LEVELS_JOB_CAPTURE_CUBE, LEVELS_JOB_CAPTURE_MAP};
use crate::viewer::LevelViewerFolder;

/// A picture a viewport draws shot by shot, put together as the shots come back: a level map's tiles, or a cubemap's
/// faces.
pub trait ViewerAssembly: Sized + Send + 'static {
  /// What the renderer is asked to draw.
  type Plan: Copy + Send + serde::Serialize + 'static;

  const KIND: JobKind;
  const NAMES: CaptureNames;

  /// Asks `renderer` to draw `plan` in `viewport`, answering what it tells as it goes.
  fn capture(renderer: &Renderer, viewport: RenderViewportId, plan: Self::Plan) -> Receiver<CaptureEvent>;

  /// An empty picture `plan` fills.
  fn of_plan(plan: Self::Plan) -> Self;

  /// Places one `shot`, refusing one of the other capture's kind.
  ///
  /// # Errors
  ///
  /// Returns an error where the shot is not of this capture, or does not fit it.
  fn place_shot(&mut self, shot: CaptureShot, drawn: RenderCapture) -> TauriResult<()>;

  /// The picture once every shot is placed.
  fn into_picture(self) -> RgbaImage;
}

impl ViewerAssembly for MapAssembly {
  type Plan = RenderMapCapture;

  const KIND: JobKind = LEVELS_JOB_CAPTURE_MAP;
  const NAMES: CaptureNames = CaptureNames {
    capture: "level map",
    shots: "tiles",
  };

  fn capture(renderer: &Renderer, viewport: RenderViewportId, plan: Self::Plan) -> Receiver<CaptureEvent> {
    renderer.capture_map(viewport, plan)
  }

  fn of_plan(plan: Self::Plan) -> Self {
    Self::new(plan.width, plan.height)
  }

  fn place_shot(&mut self, shot: CaptureShot, drawn: RenderCapture) -> TauriResult<()> {
    match shot {
      CaptureShot::Tile(tile) => self.place(&tile, drawn).map_err(|error| error.to_string()),
      CaptureShot::Face(_) => Err("A level map came back with a cubemap's face".to_string()),
    }
  }

  fn into_picture(self) -> RgbaImage {
    self.finish()
  }
}

impl ViewerAssembly for CubeAssembly {
  type Plan = RenderCubeCapture;

  const KIND: JobKind = LEVELS_JOB_CAPTURE_CUBE;
  const NAMES: CaptureNames = CaptureNames {
    capture: "cubemap",
    shots: "faces",
  };

  fn capture(renderer: &Renderer, viewport: RenderViewportId, plan: Self::Plan) -> Receiver<CaptureEvent> {
    renderer.capture_cube(viewport, plan)
  }

  fn of_plan(plan: Self::Plan) -> Self {
    Self::new(plan.size)
  }

  fn place_shot(&mut self, shot: CaptureShot, drawn: RenderCapture) -> TauriResult<()> {
    match shot {
      CaptureShot::Face(face) => self.place(&face, drawn).map_err(|error| error.to_string()),
      CaptureShot::Tile(_) => Err("A cubemap came back with a level map's tile".to_string()),
    }
  }

  fn into_picture(self) -> RgbaImage {
    self.finish()
  }
}

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
