use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;
use std::time::Instant;

use xrf_math::glam::{Mat4, Vec3};
use xrf_renderer::{CameraFrame, EngineSpace};

use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::host::world_fed_clock::WorldFedClock;
use crate::host::world_fed_view::WorldFedView;
use crate::host::world_view_feed::WorldViewFeed;

/// A camera its host stands through a [`WorldViewFeed`], read once a frame: no input or command moves it, and a
/// description of a camera takes the viewport back from the host.
#[derive(Clone)]
pub(crate) struct FedCameraController {
  feed: Arc<dyn WorldViewFeed>,
  view: WorldFedView,
}

impl Debug for FedCameraController {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FedCameraController")
      .field("view", &self.view)
      .finish_non_exhaustive()
  }
}

impl FedCameraController {
  pub(crate) fn new(feed: Arc<dyn WorldViewFeed>) -> Self {
    Self {
      view: feed.read(Instant::now()),
      feed,
    }
  }

  /// Reads the view to draw at `now`.
  pub(crate) fn update(&mut self, now: Instant) {
    self.view = self.feed.read(now);
  }

  /// The game's clock as the host gave it last, none where the host leaves the day to the viewport.
  pub(crate) fn get_clock(&self) -> Option<WorldFedClock> {
    self.view.clock
  }

  /// Where it stands and the point a metre ahead, in the renderer's space as every pose is.
  pub(crate) fn get_pose(&self) -> WorldCameraPose {
    let position: Vec3 = EngineSpace::mirror(self.view.camera.position);

    WorldCameraPose {
      position: position.to_array(),
      target: (position + EngineSpace::mirror(self.view.camera.forward)).to_array(),
    }
  }

  pub(crate) fn get_frame(&self) -> CameraFrame {
    let camera = self.view.camera;
    let position: Vec3 = EngineSpace::mirror(camera.position);
    let forward: Vec3 = EngineSpace::mirror(camera.forward).normalize_or(Vec3::NEG_Z);
    // The renderer's camera looks down its `-z`, `+x` to its right.
    let right: Vec3 = forward.cross(EngineSpace::mirror(camera.up)).normalize_or(Vec3::X);
    let up: Vec3 = right.cross(forward);
    let world: Mat4 = Mat4::from_cols(
      right.extend(0.0),
      up.extend(0.0),
      (-forward).extend(0.0),
      position.extend(1.0),
    );

    CameraFrame {
      position,
      view: world.inverse(),
      field_of_view: camera.field_of_view,
      near: camera.near,
      far: camera.far,
    }
  }
}
