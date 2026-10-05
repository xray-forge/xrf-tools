use xrf_renderer::CameraFrame;

use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::world_camera::WorldCamera;
use crate::contract::world_camera_command::WorldCameraCommand;
use crate::contract::world_camera_pose::WorldCameraPose;
use crate::contract::world_input_event::WorldInputEvent;

/// Drives a viewport's camera from what its consumer described and what the person does over it.
#[derive(Clone, Debug)]
pub enum CameraController {
  Fly(FlyCameraController),
  Orbit(OrbitCameraController),
}

impl Default for CameraController {
  fn default() -> Self {
    CameraController::Fly(FlyCameraController::default())
  }
}

impl CameraController {
  /// Takes a description, switching controllers when it asks for the other kind.
  pub fn describe(&mut self, description: WorldCamera) {
    match (self, description) {
      (CameraController::Fly(controller), WorldCamera::Fly { .. }) => {
        controller.describe(description);
      }
      (CameraController::Orbit(controller), WorldCamera::Orbit { .. }) => {
        controller.describe(description);
      }
      (this, WorldCamera::Fly { .. }) => *this = CameraController::Fly(FlyCameraController::new(description)),
      (this, WorldCamera::Orbit { .. }) => *this = CameraController::Orbit(OrbitCameraController::new(description)),
    }
  }

  pub fn command(&mut self, command: WorldCameraCommand) {
    match self {
      CameraController::Fly(controller) => controller.command(command),
      CameraController::Orbit(controller) => controller.command(command),
    }
  }

  pub fn input(&mut self, event: &WorldInputEvent) {
    match self {
      CameraController::Fly(controller) => controller.input(event),
      CameraController::Orbit(controller) => controller.input(event),
    }
  }

  /// Advances the camera by a frame's seconds; `height` is the viewport's in CSS pixels.
  pub fn update(&mut self, delta: f32, height: f32) {
    match self {
      CameraController::Fly(controller) => controller.update(delta),
      CameraController::Orbit(controller) => {
        controller.resize(height);
        controller.update(delta)
      }
    }
  }

  /// The lens's vertical field of view, in degrees.
  pub fn get_field_of_view(&self) -> f32 {
    let description: WorldCamera = match self {
      CameraController::Fly(controller) => controller.get_description(),
      CameraController::Orbit(controller) => controller.get_description(),
    };

    description.get_lens().0
  }

  pub fn get_pose(&self) -> WorldCameraPose {
    match self {
      CameraController::Fly(controller) => controller.get_pose(),
      CameraController::Orbit(controller) => controller.get_pose(),
    }
  }

  /// Where it stands and what its lens is this frame.
  pub fn get_frame(&self) -> CameraFrame {
    match self {
      CameraController::Fly(controller) => controller.get_frame(),
      CameraController::Orbit(controller) => controller.get_frame(),
    }
  }
}
