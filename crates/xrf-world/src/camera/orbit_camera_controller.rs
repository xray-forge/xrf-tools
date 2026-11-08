use std::f32::consts::{PI, TAU};
use std::time::Duration;

use glam::Vec3;
use xrf_renderer::CameraFrame;

use crate::camera::wheel_delta::WheelDelta;
use crate::contract::camera::world_camera::WorldCamera;
use crate::contract::camera::world_camera_command::WorldCameraCommand;
use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::contract::camera::world_input_buttons::WorldInputButtons;
use crate::contract::camera::world_input_event::WorldInputEvent;
use crate::contract::camera::world_input_modifiers::WorldInputModifiers;

/// Keeps the camera off the poles, where turning around the target would flip it over.
const POLE_MARGIN: f32 = 1e-4;

/// What one wheel notch of a hundred units changes the distance by: a twentieth of it.
const ZOOM_BASE: f32 = 0.95;

/// What a pixel of a dollying drag multiplies the distance by: a hundred pixels, two fifths closer.
const DOLLY_BASE: f32 = 0.995;

/// How a drag moves an orbiting camera.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OrbitDrag {
  Rotate,
  Pan,
  Dolly,
}

impl OrbitDrag {
  /// The main button alone rotates and every other drag pans, unless Alt picks the editor's own: Alt and the main
  /// button rotate, Alt and the secondary dolly, Alt and the auxiliary pan.
  fn of(buttons: WorldInputButtons, modifiers: WorldInputModifiers) -> Option<Self> {
    Some(match (modifiers.alt_key, buttons) {
      (true, WorldInputButtons::MAIN) => Self::Rotate,
      (true, WorldInputButtons::SECONDARY) => Self::Dolly,
      (true, WorldInputButtons::AUXILIARY) => Self::Pan,
      (true, _) => return None,
      (false, WorldInputButtons::MAIN) if !modifiers.shift_key && !modifiers.ctrl_key => Self::Rotate,
      (false, _) => Self::Pan,
    })
  }
}

/// A camera orbiting a target: the main button turns it around the target, the others pan, and the wheel dollies; with
/// Alt, the main button turns, the secondary dollies and the auxiliary pans.
#[derive(Clone, Debug)]
pub struct OrbitCameraController {
  description: WorldCamera,
  position: Vec3,
  target: Vec3,
  /// The viewport's height in CSS pixels, which a drag is measured against.
  height: f32,
  /// The modifiers held as the drag began, which keep what it does while its buttons change.
  modifiers: WorldInputModifiers,
  dragged: Option<OrbitDrag>,
}

impl Default for OrbitCameraController {
  fn default() -> Self {
    Self::new(WorldCamera::Orbit {
      position: [0.0, 0.0, 3.0],
      target: [0.0, 0.0, 0.0],
      field_of_view: 45.0,
      near: 0.01,
      far: 100.0,
    })
  }
}

impl OrbitCameraController {
  pub fn new(description: WorldCamera) -> Self {
    let (position, target) = description.get_start();

    Self {
      description,
      position: Vec3::from_array(position),
      target: Vec3::from_array(target),
      height: 1.0,
      modifiers: WorldInputModifiers::default(),
      dragged: None,
    }
  }

  /// Takes a new description, answering whether the camera jumped to its start rather than keeping where it stands.
  pub fn describe(&mut self, description: WorldCamera) -> bool {
    let is_moved: bool = description.get_start() != self.description.get_start();

    self.description = description;

    if is_moved {
      self.reset();
    }

    is_moved
  }

  pub fn command(&mut self, command: WorldCameraCommand) {
    match command {
      WorldCameraCommand::Reset => self.reset(),
      WorldCameraCommand::Dolly { step } => self.dolly(step),
    }
  }

  /// The viewport's height in CSS pixels, against which drags are measured.
  pub fn resize(&mut self, height: f32) {
    self.height = height.max(1.0);
  }

  pub fn input(&mut self, event: &WorldInputEvent) {
    match event {
      WorldInputEvent::DragStart { buttons, modifiers } => {
        self.modifiers = *modifiers;
        self.dragged = OrbitDrag::of(*buttons, *modifiers);
      }
      WorldInputEvent::DragButtons { buttons } => self.dragged = OrbitDrag::of(*buttons, self.modifiers),
      WorldInputEvent::DragMove { dx, dy } => match self.dragged {
        Some(OrbitDrag::Rotate) => self.rotate(*dx, *dy),
        Some(OrbitDrag::Pan) => self.pan(*dx, *dy),
        // Dragged right or up, it closes in.
        Some(OrbitDrag::Dolly) => self.dolly(DOLLY_BASE.powf(*dx - *dy)),
        None => {}
      },
      WorldInputEvent::DragEnd { .. } | WorldInputEvent::ReleaseAll => self.dragged = None,
      WorldInputEvent::Wheel {
        delta_y, delta_mode, ..
      } => {
        let pixels: f32 = WheelDelta::to_pixels(*delta_y, *delta_mode, self.height);
        let scale: f32 = ZOOM_BASE.powf((pixels * 0.01).abs());

        self.dolly(if pixels > 0.0 { 1.0 / scale } else { scale });
      }
      WorldInputEvent::Hover { .. }
      | WorldInputEvent::KeyDown { .. }
      | WorldInputEvent::KeyUp { .. }
      | WorldInputEvent::ContextMenu => {}
    }
  }

  pub fn update(&mut self, _delta: Duration) {}

  pub fn get_description(&self) -> WorldCamera {
    self.description
  }

  pub fn get_pose(&self) -> WorldCameraPose {
    WorldCameraPose {
      position: self.position.to_array(),
      target: self.target.to_array(),
    }
  }

  /// Where it stands and what its lens is this frame.
  pub fn get_frame(&self) -> CameraFrame {
    let (field_of_view, near, far) = self.description.get_lens();

    CameraFrame {
      position: self.position,
      view: glam::camera::rh::view::look_at_mat4(self.position, self.target, Vec3::Y),
      field_of_view,
      near,
      far,
    }
  }

  fn reset(&mut self) {
    let (position, target) = self.description.get_start();

    self.position = Vec3::from_array(position);
    self.target = Vec3::from_array(target);
  }

  /// Turns around the target: a drag across the whole viewport height is a full turn.
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
