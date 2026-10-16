use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::time::Duration;

use glam::{EulerRot, Mat4, Quat, Vec3};
use xrf_renderer::CameraFrame;

use crate::camera::fly_key::FlyKey;
use crate::contract::world_camera::WorldCamera;
use crate::contract::world_camera_command::WorldCameraCommand;
use crate::contract::world_camera_pose::WorldCameraPose;
use crate::contract::world_input_event::WorldInputEvent;

/// Just short of straight up, so looking at the sky never flips the horizon over.
const MAX_PITCH: f32 = FRAC_PI_2 - 0.001;

/// Radians a second a held arrow key turns the camera by: an eighth of a turn.
const TURN_SPEED: f32 = FRAC_PI_4;

/// The longest step one frame moves by, so a frame after a stall does not throw the camera across the level.
const MAX_DELTA: Duration = Duration::from_millis(250);

/// A free camera: a drag or the arrow keys turn it, the other keys move it along where it faces, rising along its own
/// up as it is pitched. It holds yaw and pitch itself, since a rotation read back cannot tell `+π` from `-π`.
#[derive(Clone, Debug)]
pub struct FlyCameraController {
  description: WorldCamera,
  position: Vec3,
  yaw: f32,
  pitch: f32,
  held: HashSet<FlyKey>,
  /// Pointer movement gathered since the last frame, which is what applies it.
  look: (f32, f32),
  /// Whether a drag of the main button is turning the camera.
  is_looking: bool,
}

impl Default for FlyCameraController {
  fn default() -> Self {
    Self::new(WorldCamera::Fly {
      position: [0.0, 2.0, 0.0],
      target: [0.0, 2.0, -1.0],
      field_of_view: 67.5,
      near: 0.2,
      far: 10_000.0,
      speed: 10.0,
      boost: 4.0,
      sensitivity: 0.003,
    })
  }
}

impl FlyCameraController {
  pub fn new(description: WorldCamera) -> Self {
    let mut controller: Self = Self {
      description,
      position: Vec3::ZERO,
      yaw: 0.0,
      pitch: 0.0,
      held: HashSet::new(),
      look: (0.0, 0.0),
      is_looking: false,
    };

    controller.reset();
    controller
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
    if command == WorldCameraCommand::Reset {
      self.reset();
    }
  }

  pub fn input(&mut self, event: &WorldInputEvent) {
    match event {
      // The main button alone: another one would fight it.
      WorldInputEvent::DragStart { button, .. } => self.is_looking = *button == 0,
      WorldInputEvent::DragMove { dx, dy } => {
        if self.is_looking {
          self.look.0 += *dx;
          self.look.1 += *dy;
        }
      }
      WorldInputEvent::DragEnd { .. } => self.is_looking = false,
      WorldInputEvent::KeyDown { code } => {
        if let Some(key) = FlyKey::from_code(code) {
          self.held.insert(key);
        }
      }
      WorldInputEvent::KeyUp { code } => {
        if let Some(key) = FlyKey::from_code(code) {
          self.held.remove(&key);
        }
      }
      // A key whose release will never arrive would otherwise fly the camera away unattended.
      WorldInputEvent::ReleaseAll => {
        self.held.clear();
        self.is_looking = false;
      }
      WorldInputEvent::Hover { .. } | WorldInputEvent::Wheel { .. } | WorldInputEvent::ContextMenu => {}
    }
  }

  /// Advances the camera by a frame's time.
  pub fn update(&mut self, delta: Duration) {
    let WorldCamera::Fly {
      speed,
      boost,
      sensitivity,
      ..
    } = self.description
    else {
      return;
    };

    let step: f32 = delta.min(MAX_DELTA).as_secs_f32();
    let turned: f32 = TURN_SPEED * step;

    // Looking before moving, because where the camera walks is where it faces.
    self.yaw += self.axis(FlyKey::TurnLeft, FlyKey::TurnRight) * turned - self.look.0 * sensitivity;
    self.pitch = (self.pitch + self.axis(FlyKey::LookUp, FlyKey::LookDown) * turned - self.look.1 * sensitivity)
      .clamp(-MAX_PITCH, MAX_PITCH);
    self.look = (0.0, 0.0);

    let forward: f32 = self.axis(FlyKey::Forward, FlyKey::Back);
    let strafe: f32 = self.axis(FlyKey::Right, FlyKey::Left);
    let rise: f32 = self.axis(FlyKey::Up, FlyKey::Down);

    if forward == 0.0 && strafe == 0.0 && rise == 0.0 {
      return;
    }

    let distance: f32 = speed * if self.held.contains(&FlyKey::Fast) { boost } else { 1.0 } * step;
    let rotation: Quat = self.get_rotation();

    self.position += rotation * Vec3::NEG_Z * (forward * distance)
      + rotation * Vec3::X * (strafe * distance)
      + rotation * Vec3::Y * (rise * distance);
  }

  /// Whether a key or a drag is moving the camera.
  pub fn is_moving(&self) -> bool {
    !self.held.is_empty() || self.is_looking
  }

  pub fn get_description(&self) -> WorldCamera {
    self.description
  }

  pub fn get_pose(&self) -> WorldCameraPose {
    let target: Vec3 = self.position + self.get_rotation() * Vec3::NEG_Z;

    WorldCameraPose {
      position: self.position.to_array(),
      target: target.to_array(),
    }
  }

  /// The view through the lens, its far plane no farther than a limit.
  /// Where it stands and what its lens is this frame.
  pub fn get_frame(&self) -> CameraFrame {
    let (field_of_view, near, far) = self.description.get_lens();
    let world: Mat4 = Mat4::from_rotation_translation(self.get_rotation(), self.position);

    CameraFrame {
      position: self.position,
      view: world.inverse(),
      field_of_view,
      near,
      far,
    }
  }

  /// Back to the start, looking at its target.
  fn reset(&mut self) {
    let (position, target) = self.description.get_start();
    let position: Vec3 = Vec3::from_array(position);
    let direction: Vec3 = Vec3::from_array(target) - position;

    self.position = position;

    // A camera told to look at where it stands keeps whatever it was pointing at.
    if direction.length_squared() > 0.0 {
      self.yaw = (-direction.x).atan2(-direction.z);
      self.pitch = direction.y.atan2(direction.x.hypot(direction.z));
    }
  }

  fn get_rotation(&self) -> Quat {
    Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
  }

  fn axis(&self, positive: FlyKey, negative: FlyKey) -> f32 {
    (self.held.contains(&positive) as i32 - self.held.contains(&negative) as i32) as f32
  }
}
