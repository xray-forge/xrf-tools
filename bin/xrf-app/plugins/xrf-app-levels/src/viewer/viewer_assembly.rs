use std::sync::mpsc::Receiver;

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_app_core::jobs::JobKind;
use xrf_app_render::{CaptureNames, Renderer};
use xrf_renderer::{
  CaptureEvent, CaptureShot, CubeAssembly, MapAssembly, RenderCapture, RenderCubeCapture, RenderMapCapture,
  RenderViewportId,
};

use crate::ipc::levels::{LEVELS_JOB_CAPTURE_CUBE, LEVELS_JOB_CAPTURE_MAP};

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
