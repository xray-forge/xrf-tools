use std::time::Duration;

use xrf_renderer::CameraFrame;

use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::camera::world_camera::WorldCamera;
use crate::contract::camera::world_camera_command::WorldCameraCommand;
use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::contract::camera::world_input_event::WorldInputEvent;

/// Drives a viewport's camera from what its consumer described and what the person does over it.
#[derive(Clone, Debug)]
pub(crate) enum CameraController {
  Fly(FlyCameraController),
  Orbit(OrbitCameraController),
}

impl Default for CameraController {
  fn default() -> Self {
    Self::Fly(FlyCameraController::default())
  }
}

impl CameraController {
  /// Takes a description, switching controllers when it asks for the other kind.
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

  pub(crate) fn command(&mut self, command: WorldCameraCommand) {
    match self {
      Self::Fly(controller) => controller.command(command),
      Self::Orbit(controller) => controller.command(command),
    }
  }

  pub(crate) fn input(&mut self, event: &WorldInputEvent) {
    match self {
      Self::Fly(controller) => controller.input(event),
      Self::Orbit(controller) => controller.input(event),
    }
  }

  /// Advances the camera by a frame's time; `height` is the viewport's in CSS pixels.
  pub(crate) fn update(&mut self, delta: Duration, height: f32) {
    match self {
      Self::Fly(controller) => {
        controller.resize(height);
        controller.update(delta);
      }
      Self::Orbit(controller) => {
        controller.resize(height);
        controller.update(delta);
      }
    }
  }

  /// The steps of speed the wheel asked for since they were last taken, positive faster; an orbit camera has none.
  pub(crate) fn take_speed_steps(&mut self) -> i32 {
    match self {
      Self::Fly(controller) => controller.take_speed_steps(),
      Self::Orbit(_) => 0,
    }
  }

  pub(crate) fn get_pose(&self) -> WorldCameraPose {
    match self {
      Self::Fly(controller) => controller.get_pose(),
      Self::Orbit(controller) => controller.get_pose(),
    }
  }

  /// Where it stands and what its lens is this frame.
  pub(crate) fn get_frame(&self) -> CameraFrame {
    match self {
      Self::Fly(controller) => controller.get_frame(),
      Self::Orbit(controller) => controller.get_frame(),
    }
  }
}
