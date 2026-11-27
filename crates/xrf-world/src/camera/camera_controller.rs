use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer::CameraFrame;

use crate::camera::fed_camera_controller::FedCameraController;
use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::camera::world_camera::WorldCamera;
use crate::contract::camera::world_camera_command::WorldCameraCommand;
use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::contract::camera::world_input_event::WorldInputEvent;
use crate::host::world_fed_clock::WorldFedClock;
use crate::host::world_view_feed::WorldViewFeed;

/// Drives a viewport's camera from what its consumer described and what the person does over it, or from its host's
/// feed.
#[derive(Clone, Debug)]
pub(crate) enum CameraController {
  Fly(FlyCameraController),
  Orbit(OrbitCameraController),
  Fed(FedCameraController),
}

impl Default for CameraController {
  fn default() -> Self {
    Self::Fly(FlyCameraController::default())
  }
}

impl CameraController {
  /// Takes a description, switching controllers when it asks for the other kind; a fed camera is given back.
  pub(crate) fn describe(&mut self, description: WorldCamera) {
    match (self, description) {
      (Self::Fly(controller), WorldCamera::Fly { .. }) => {
        controller.describe(description);
      }
      (Self::Orbit(controller), WorldCamera::Orbit { .. }) => {
        controller.describe(description);
      }
      (this, WorldCamera::Fly { .. }) => *this = Self::Fly(FlyCameraController::new(description)),
      (this, WorldCamera::Orbit { .. }) => *this = Self::Orbit(OrbitCameraController::new(description)),
    }
  }

  /// Hands the camera to its host's feed until a description takes it back.
  pub(crate) fn feed(&mut self, feed: Arc<dyn WorldViewFeed>) {
    *self = Self::Fed(FedCameraController::new(feed));
  }

  pub(crate) fn command(&mut self, command: WorldCameraCommand) {
    match self {
      Self::Fly(controller) => controller.command(command),
      Self::Orbit(controller) => controller.command(command),
      Self::Fed(_) => {}
    }
  }

  pub(crate) fn input(&mut self, event: &WorldInputEvent) {
    match self {
      Self::Fly(controller) => controller.input(event),
      Self::Orbit(controller) => controller.input(event),
      Self::Fed(_) => {}
    }
  }

  /// Advances the camera to a frame at `now`, `delta` after the last; `height` is the viewport's in CSS pixels.
  pub(crate) fn update(&mut self, now: Instant, delta: Duration, height: f32) {
    match self {
      Self::Fly(controller) => {
        controller.resize(height);
        controller.update(delta);
      }
      Self::Orbit(controller) => {
        controller.resize(height);
        controller.update(delta);
      }
      Self::Fed(controller) => controller.update(now),
    }
  }

  /// The steps of speed the wheel asked for since they were last taken, positive faster; only a flying camera has any.
  pub(crate) fn take_speed_steps(&mut self) -> i32 {
    match self {
      Self::Fly(controller) => controller.take_speed_steps(),
      Self::Orbit(_) | Self::Fed(_) => 0,
    }
  }

  /// The game's clock a fed camera's host runs, none for the world's own cameras.
  pub(crate) fn get_clock(&self) -> Option<WorldFedClock> {
    match self {
      Self::Fed(controller) => controller.get_clock(),
      Self::Fly(_) | Self::Orbit(_) => None,
    }
  }

  pub(crate) fn get_pose(&self) -> WorldCameraPose {
    match self {
      Self::Fly(controller) => controller.get_pose(),
      Self::Orbit(controller) => controller.get_pose(),
      Self::Fed(controller) => controller.get_pose(),
    }
  }

  /// Where it stands and what its lens is this frame.
  pub(crate) fn get_frame(&self) -> CameraFrame {
    match self {
      Self::Fly(controller) => controller.get_frame(),
      Self::Orbit(controller) => controller.get_frame(),
      Self::Fed(controller) => controller.get_frame(),
    }
  }
}
