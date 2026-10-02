use std::f32::consts::{PI, TAU};

use glam::Vec3;

use crate::camera::camera_view::CameraView;
use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_camera_pose::RenderCameraPose;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_input_kind::RenderInputKind;

/// Keeps the camera off the poles, where turning around the target would flip it over.
const POLE_MARGIN: f32 = 1e-4;

/// What one wheel notch of a hundred units changes the distance by, as three's `OrbitControls` dollies.
const ZOOM_BASE: f32 = 0.95;

/// How a drag moves an orbiting camera.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OrbitDrag {
  Rotate,
  Pan,
}

/// A camera orbiting a target: the main button turns it around the target, the others pan, and the wheel dollies.
#[derive(Clone, Debug)]
pub struct OrbitCameraController {
  description: RenderCamera,
  position: Vec3,
  target: Vec3,
  /// The viewport's height in CSS pixels, which a drag is measured against.
  height: f32,
  dragged: Option<(i32, f32, f32, OrbitDrag)>,
}

impl Default for OrbitCameraController {
  fn default() -> Self {
    Self::new(RenderCamera::Orbit {
      position: [0.0, 0.0, 3.0],
      target: [0.0, 0.0, 0.0],
      field_of_view: 45.0,
      near: 0.01,
      far: 100.0,
    })
  }
}

impl OrbitCameraController {
  pub fn new(description: RenderCamera) -> Self {
    let (position, target) = description.get_start();

    Self {
      description,
      position: Vec3::from_array(position),
      target: Vec3::from_array(target),
      height: 1.0,
      dragged: None,
    }
  }

  /// Takes a new description, answering whether the camera jumped to its start rather than keeping where it stands.
  pub fn describe(&mut self, description: RenderCamera) -> bool {
    let is_moved: bool = description.get_start() != self.description.get_start();

    self.description = description;

    if is_moved {
      self.reset();
    }

    is_moved
  }

  pub fn command(&mut self, command: RenderCameraCommand) {
    match command {
      RenderCameraCommand::Reset => self.reset(),
      RenderCameraCommand::Dolly { step } => self.dolly(step),
    }
  }

  /// The viewport's height in CSS pixels, against which drags are measured.
  pub fn resize(&mut self, height: f32) {
    self.height = height.max(1.0);
  }

  pub fn input(&mut self, event: &RenderInputEvent) {
    match event.kind {
      RenderInputKind::PointerDown if event.is_primary => {
        let drag: OrbitDrag = if event.button == 0 && !event.shift_key && !event.ctrl_key {
          OrbitDrag::Rotate
        } else {
          OrbitDrag::Pan
        };

        self.dragged = Some((event.pointer_id, event.x, event.y, drag));
      }
      RenderInputKind::PointerMove => {
        if let Some((id, x, y, drag)) = self.dragged
          && id == event.pointer_id
        {
          match drag {
            OrbitDrag::Rotate => self.rotate(event.x - x, event.y - y),
            OrbitDrag::Pan => self.pan(event.x - x, event.y - y),
          }

          self.dragged = Some((id, event.x, event.y, drag));
        }
      }
      RenderInputKind::PointerUp | RenderInputKind::PointerCancel => {
        if self.dragged.is_some_and(|(id, ..)| id == event.pointer_id) {
          self.dragged = None;
        }
      }
      RenderInputKind::Wheel => {
        // A line or a page of wheel is worth what a browser scrolls for it, so every mouse dollies alike.
        let pixels: f32 = match event.delta_mode {
          1 => event.delta_y * 16.0,
          2 => event.delta_y * self.height,
          _ => event.delta_y,
        };
        let scale: f32 = ZOOM_BASE.powf((pixels * 0.01).abs());

        self.dolly(if pixels > 0.0 { 1.0 / scale } else { scale });
      }
      RenderInputKind::Blur => self.dragged = None,
      _ => {}
    }
  }

  pub fn update(&mut self, _delta: f32) {}

  pub fn is_moving(&self) -> bool {
    self.dragged.is_some()
  }

  pub fn get_pose(&self) -> RenderCameraPose {
    RenderCameraPose {
      position: self.position.to_array(),
      target: self.target.to_array(),
    }
  }

  pub fn get_view(&self, aspect: f32) -> CameraView {
    let (field_of_view, near, far) = self.description.get_lens();

    CameraView::new(
      self.position,
      glam::camera::rh::view::look_at_mat4(self.position, self.target, Vec3::Y),
      field_of_view,
      aspect,
      near,
      far,
    )
  }

  fn reset(&mut self) {
    let (position, target) = self.description.get_start();

    self.position = Vec3::from_array(position);
    self.target = Vec3::from_array(target);
  }

  /// Turns around the target: a drag across the whole viewport height is a full turn, as three's controls turn.
  fn rotate(&mut self, dx: f32, dy: f32) {
    let offset: Vec3 = self.position - self.target;
    let radius: f32 = offset.length();

    if radius == 0.0 {
      return;
    }

    let theta: f32 = offset.x.atan2(offset.z) - TAU * dx / self.height;
    let phi: f32 =
      ((offset.y / radius).clamp(-1.0, 1.0).acos() - TAU * dy / self.height).clamp(POLE_MARGIN, PI - POLE_MARGIN);

    self.position = self.target
      + Vec3::new(
        radius * phi.sin() * theta.sin(),
        radius * phi.cos(),
        radius * phi.sin() * theta.cos(),
      );
  }

  /// Moves the camera and its target together, so what was under the pointer stays under it.
  fn pan(&mut self, dx: f32, dy: f32) {
    let (field_of_view, ..) = self.description.get_lens();
    let offset: Vec3 = self.position - self.target;
    let per_pixel: f32 = 2.0 * offset.length() * (field_of_view.to_radians() / 2.0).tan() / self.height;
    let forward: Vec3 = (-offset).normalize_or_zero();
    let right: Vec3 = forward.cross(Vec3::Y).normalize_or_zero();
    let up: Vec3 = right.cross(forward);
    let shift: Vec3 = right * (-dx * per_pixel) + up * (dy * per_pixel);

    self.position += shift;
    self.target += shift;
  }

  /// Towards the target or away from it, by a multiplier on the distance: above one moves away.
  fn dolly(&mut self, step: f32) {
    let offset: Vec3 = self.position - self.target;

    // A camera sitting exactly on what it orbits has no direction to be moved along.
    if offset.length_squared() > 0.0 && step > 0.0 {
      self.position = self.target + offset * step;
    }
  }
}
