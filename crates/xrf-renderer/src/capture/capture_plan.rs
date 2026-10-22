use xrf_error::{XrfError, XrfResult};

use crate::camera::camera_frame::CameraFrame;
use crate::capture::capture_shot::CaptureShot;
use crate::capture::cube_face::CubeFace;
use crate::capture::map_tile_plan::MapTilePlan;
use crate::contract::render_cube_capture::{
  RENDER_CUBE_MOST_PIXELS, RENDER_CUBE_MOST_SUPERSAMPLING, RenderCubeCapture,
};
use crate::contract::render_map_capture::RenderMapCapture;

/// What a capture draws: views metered in turn until their exposure settles, then each of its shots in order, the
/// exposure held for all of them.
#[derive(Clone, Debug, PartialEq)]
pub struct CapturePlan {
  pub shots: Vec<CaptureShot>,
  /// The views its exposure is metered over, one a frame in turn.
  pub meters: Vec<CaptureShot>,
  /// Frames metered before the first shot.
  pub meter_frames: u32,
  /// Frames each shot is drawn from where it stands before the one kept.
  pub settle_frames: u32,
}

impl CapturePlan {
  /// Frames a map meters its whole area, as long as a fast adaptation takes to settle.
  pub const MAP_METER_FRAMES: u32 = 60;

  /// Frames a cubemap's faces are metered in turn: forty rounds of six, as long as the slow adaptation over all six
  /// takes to settle.
  pub const CUBE_METER_FRAMES: u32 = 240;

  /// Pixels a cubemap's face is metered at on a side.
  pub const CUBE_METER: u32 = 128;

  /// A level map's tiles, row by row from its top left, after a view of the whole area.
  ///
  /// # Errors
  ///
  /// Returns an error where [`MapTilePlan::new`] does.
  pub fn new_map(capture: RenderMapCapture, most_texture: u32) -> XrfResult<Self> {
    let plan: MapTilePlan = MapTilePlan::new(capture, most_texture)?;

    Ok(Self {
      meters: vec![CaptureShot::Tile(plan.get_meter())],
      shots: plan.tiles.iter().copied().map(CaptureShot::Tile).collect(),
      meter_frames: Self::MAP_METER_FRAMES,
      settle_frames: capture.settle_frames,
    })
  }

  /// A cubemap's six faces around `camera`, its far plane no farther than `far_limit`, for a device whose textures are
  /// at most `most_texture` pixels a side.
  ///
  /// # Errors
  ///
  /// Returns an error for a face or supersampling past what is drawn.
  pub fn new_cube(
    capture: RenderCubeCapture,
    camera: &CameraFrame,
    far_limit: f32,
    most_texture: u32,
  ) -> XrfResult<Self> {
    let RenderCubeCapture {
      size,
      supersampling,
      settle_frames,
    } = capture;

    if size == 0 || size > RENDER_CUBE_MOST_PIXELS {
      return Err(XrfError::new_invalid_error(format!(
        "A cubemap's face must be 1 to {RENDER_CUBE_MOST_PIXELS} pixels a side, not {size}"
      )));
    }

    if supersampling == 0 || supersampling > RENDER_CUBE_MOST_SUPERSAMPLING {
      return Err(XrfError::new_invalid_error(format!(
        "A cubemap is drawn 1 to {RENDER_CUBE_MOST_SUPERSAMPLING} times over, not {supersampling}"
      )));
    }

    if size * supersampling > most_texture {
      return Err(XrfError::new_invalid_error(format!(
        "A face drawn {supersampling} times over takes {} pixels, past the {most_texture} the GPU draws",
        size * supersampling
      )));
    }

    let face = |index: usize, size: u32, supersampling: u32| {
      CaptureShot::Face(CubeFace {
        index,
        position: camera.position,
        near: camera.near,
        far: camera.far.min(far_limit).max(camera.near * 2.0),
        size,
        supersampling,
      })
    };

    Ok(Self {
      shots: (0..CubeFace::COUNT)
        .map(|index| face(index, size, supersampling))
        .collect(),
      meters: (0..CubeFace::COUNT)
        .map(|index| face(index, Self::CUBE_METER.min(size), 1))
        .collect(),
      meter_frames: Self::CUBE_METER_FRAMES,
      settle_frames,
    })
  }

  /// The view metered on the `frame`th metering frame.
  pub fn get_meter(&self, frame: u32) -> Option<CaptureShot> {
    self.meters.get(frame as usize % self.meters.len().max(1)).copied()
  }
}
