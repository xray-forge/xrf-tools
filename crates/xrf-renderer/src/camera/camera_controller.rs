use crate::camera::camera_view::CameraView;
use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_camera_pose::RenderCameraPose;
use crate::contract::render_input_event::RenderInputEvent;

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
  pub fn describe(&mut self, description: RenderCamera) {
    match (self, description) {
      (CameraController::Fly(controller), RenderCamera::Fly { .. }) => {
        controller.describe(description);
      }
      (CameraController::Orbit(controller), RenderCamera::Orbit { .. }) => {
        controller.describe(description);
      }
      (this, RenderCamera::Fly { .. }) => *this = CameraController::Fly(FlyCameraController::new(description)),
      (this, RenderCamera::Orbit { .. }) => *this = CameraController::Orbit(OrbitCameraController::new(description)),
    }
  }

  pub fn command(&mut self, command: RenderCameraCommand) {
    match self {
      CameraController::Fly(controller) => controller.command(command),
      CameraController::Orbit(controller) => controller.command(command),
    }
  }

  pub fn input(&mut self, event: &RenderInputEvent) {
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
    let description: RenderCamera = match self {
      CameraController::Fly(controller) => controller.get_description(),
      CameraController::Orbit(controller) => controller.get_description(),
    };

    description.get_lens().0
  }

  pub fn get_pose(&self) -> RenderCameraPose {
    match self {
      CameraController::Fly(controller) => controller.get_pose(),
      CameraController::Orbit(controller) => controller.get_pose(),
    }
  }

  pub fn get_view(&self, aspect: f32, far_limit: f32) -> CameraView {
    match self {
      CameraController::Fly(controller) => controller.get_view(aspect, far_limit),
      CameraController::Orbit(controller) => controller.get_view(aspect, far_limit),
    }
  }
}
