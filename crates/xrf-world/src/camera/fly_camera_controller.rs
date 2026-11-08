use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::time::Duration;

use glam::{EulerRot, Mat4, Quat, Vec3};
use xrf_renderer::CameraFrame;

use crate::camera::fly_key::FlyKey;
use crate::camera::wheel_delta::WheelDelta;
use crate::contract::camera::world_camera::WorldCamera;
use crate::contract::camera::world_camera_command::WorldCameraCommand;
use crate::contract::camera::world_camera_pose::WorldCameraPose;
use crate::contract::camera::world_input_buttons::WorldInputButtons;
use crate::contract::camera::world_input_event::WorldInputEvent;
use crate::contract::camera::world_input_modifiers::WorldInputModifiers;

/// Just short of straight up, so looking at the sky never flips the horizon over.
const MAX_PITCH: f32 = FRAC_PI_2 - 0.001;

/// Radians a second a held arrow key turns the camera by: an eighth of a turn.
const TURN_SPEED: f32 = FRAC_PI_4;

/// The longest step one frame moves by, so a frame after a stall does not throw the camera across the level.
const MAX_DELTA: Duration = Duration::from_millis(250);

/// Seconds of flight a pixel of a walking or panning drag is worth: a hundred pixels go as far as a second of a key.
const DRAG_TRAVEL: f32 = 0.01;

/// Seconds of flight a pixel of wheel is worth: a notch of a hundred pixels, a quarter of a second.
const WHEEL_TRAVEL: f32 = 0.0025;

/// Pixels of wheel a step of speed is worth while flying: a mouse's notch.
const SPEED_WHEEL: f32 = 100.0;

/// What a pixel of a dollying drag multiplies the distance to the pivot by: a hundred pixels, two fifths closer.
const DOLLY_BASE: f32 = 0.995;

/// The nearest the pivot comes, so a camera moved towards it stops short rather than passing through it.
const MIN_FOCUS: f32 = 0.25;

/// What a drag does to a flying camera, by the buttons it holds and whether Alt was held as it began.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FlyDrag {
  /// The secondary button: looks around, and the movement keys fly.
  Fly,
  /// The main button: forward and back along the ground, turning left and right.
  Walk,
  /// The main and secondary buttons together, or the auxiliary: sideways along the ground, and up and down.
  Pan,
  /// Alt and the main button: turns around the pivot.
  Orbit,
  /// Alt and the secondary button: towards the pivot and away from it.
  Dolly,
  /// Alt and the auxiliary button: carries the camera and its pivot across the view.
  Track,
}

impl FlyDrag {
  /// What a drag holding these buttons does, or none for buttons nothing is mapped to.
  fn of(buttons: WorldInputButtons, modifiers: WorldInputModifiers) -> Option<Self> {
    Some(match (modifiers.alt_key, buttons) {
      (false, WorldInputButtons::SECONDARY) => Self::Fly,
      (false, WorldInputButtons::MAIN) => Self::Walk,
      (false, WorldInputButtons::MAIN_AND_SECONDARY | WorldInputButtons::AUXILIARY) => Self::Pan,
      (true, WorldInputButtons::MAIN) => Self::Orbit,
      (true, WorldInputButtons::SECONDARY) => Self::Dolly,
      (true, WorldInputButtons::AUXILIARY) => Self::Track,
      _ => return None,
    })
  }
}

/// A free camera driven as an editor viewport is: the secondary button held looks around and lets the keys fly it, the
/// wheel then asking for its speed, the main button walks it along the ground, both or the auxiliary pan it, the wheel
/// moves it forward, and with Alt it turns around, dollies to and tracks a pivot ahead of it. It holds yaw and pitch itself, since a rotation read back
/// cannot tell `+π` from `-π`.
#[derive(Clone, Debug)]
pub struct FlyCameraController {
  description: WorldCamera,
  position: Vec3,
  yaw: f32,
  pitch: f32,
  /// How far ahead the pivot stands: as far as the target it was last described or reset with, carried as it flies.
  focus: f32,
  /// The viewport's height in CSS pixels, which a track is measured against.
  height: f32,
  held: HashSet<FlyKey>,
  /// The modifiers held as the drag began, which keep what it does while its buttons change.
  modifiers: WorldInputModifiers,
  /// What the drag going on does, while one does.
  drag: Option<FlyDrag>,
  /// Pixels of wheel turned towards the person while flying, short of a whole step of speed.
  wheeled: f32,
  /// Steps of speed asked for and not yet taken, positive faster.
  speed_steps: i32,
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
      focus: 1.0,
      height: 1.0,
      held: HashSet::new(),
      modifiers: WorldInputModifiers::default(),
      drag: None,
      wheeled: 0.0,
      speed_steps: 0,
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

  /// The viewport's height in CSS pixels, against which a track is measured.
  pub fn resize(&mut self, height: f32) {
    self.height = height.max(1.0);
  }

  pub fn input(&mut self, event: &WorldInputEvent) {
    match event {
      WorldInputEvent::DragStart { buttons, modifiers } => {
        self.modifiers = *modifiers;
        self.drag = FlyDrag::of(*buttons, *modifiers);
        self.wheeled = 0.0;
      }
      WorldInputEvent::DragButtons { buttons } => self.drag = FlyDrag::of(*buttons, self.modifiers),
      WorldInputEvent::DragMove { dx, dy } => self.drag_by(*dx, *dy),
      WorldInputEvent::DragEnd { .. } => self.drag = None,
      WorldInputEvent::Wheel {
        delta_y, delta_mode, ..
      } => {
        let pixels: f32 = WheelDelta::to_pixels(*delta_y, *delta_mode, self.height);

        // While the secondary button flies the camera, the wheel asks for its speed rather than moving it.
        if self.drag == Some(FlyDrag::Fly) {
          self.step_speed(pixels);
        } else {
          self.wheel(pixels);
        }
      }
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
        self.drag = None;
      }
      WorldInputEvent::Hover { .. } | WorldInputEvent::ContextMenu => {}
    }
  }

  /// The steps of speed the wheel asked for since they were last taken, positive faster. The speed is the page's
  /// setting, which describes the camera again with the speed it chose.
  pub fn take_speed_steps(&mut self) -> i32 {
    std::mem::take(&mut self.speed_steps)
  }

  /// Advances the camera by a frame's time.
  pub fn update(&mut self, delta: Duration) {
    let step: f32 = delta.min(MAX_DELTA).as_secs_f32();
    let turned: f32 = TURN_SPEED * step;

    // Turning before moving, because where the camera flies is where it faces.
    self.turn(
      self.axis(FlyKey::TurnLeft, FlyKey::TurnRight) * turned,
      self.axis(FlyKey::LookUp, FlyKey::LookDown) * turned,
    );

    // The movement keys are heard all along, so one pressed first flies as soon as the secondary button goes down.
    if self.drag != Some(FlyDrag::Fly) {
      return;
    }

    let forward: f32 = self.axis(FlyKey::Forward, FlyKey::Back);
    let strafe: f32 = self.axis(FlyKey::Right, FlyKey::Left);
    let rise: f32 = self.axis(FlyKey::Up, FlyKey::Down);

    if forward == 0.0 && strafe == 0.0 && rise == 0.0 {
      return;
    }

    let distance: f32 = self.get_speed() * step;
    let rotation: Quat = self.get_rotation();

    self.position += rotation * Vec3::NEG_Z * (forward * distance)
      + rotation * Vec3::X * (strafe * distance)
      + Vec3::Y * (rise * distance);
  }

  /// Whether a key or a drag is held.
  #[cfg(test)]
  pub fn is_moving(&self) -> bool {
    !self.held.is_empty() || self.drag.is_some()
  }

  pub fn get_pose(&self) -> WorldCameraPose {
    let target: Vec3 = self.position + self.get_rotation() * Vec3::NEG_Z;

    WorldCameraPose {
      position: self.position.to_array(),
      target: target.to_array(),
    }
  }

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

  /// Back to the start, looking at its target, which becomes the pivot.
  fn reset(&mut self) {
    let (position, target) = self.description.get_start();
    let position: Vec3 = Vec3::from_array(position);
    let direction: Vec3 = Vec3::from_array(target) - position;

    self.position = position;

    // A camera told to look at where it stands keeps whatever it was pointing at.
    if direction.length_squared() > 0.0 {
      self.yaw = (-direction.x).atan2(-direction.z);
      self.pitch = direction.y.atan2(direction.x.hypot(direction.z));
      self.focus = direction.length().max(MIN_FOCUS);
    }
  }

  fn drag_by(&mut self, dx: f32, dy: f32) {
    let Some(drag) = self.drag else {
      return;
    };
    let sensitivity: f32 = self.get_sensitivity();

    match drag {
      FlyDrag::Fly => self.turn(-dx * sensitivity, -dy * sensitivity),
      FlyDrag::Walk => {
        self.turn(-dx * sensitivity, 0.0);
        // Pushed away, the pointer walks forward.
        self.position += self.get_heading() * Vec3::NEG_Z * (-dy * self.get_speed() * DRAG_TRAVEL);
      }
      FlyDrag::Pan => {
        let travel: f32 = self.get_speed() * DRAG_TRAVEL;

        self.position += self.get_heading() * Vec3::X * (dx * travel) + Vec3::Y * (-dy * travel);
      }
      FlyDrag::Orbit => {
        let pivot: Vec3 = self.get_pivot();

        self.turn(-dx * sensitivity, -dy * sensitivity);
        self.position = pivot - self.get_rotation() * Vec3::NEG_Z * self.focus;
      }
      FlyDrag::Dolly => {
        let pivot: Vec3 = self.get_pivot();

        // Dragged right or up, it closes in.
        self.focus = (self.focus * DOLLY_BASE.powf(dx - dy)).max(MIN_FOCUS);
        self.position = pivot - self.get_rotation() * Vec3::NEG_Z * self.focus;
      }
      FlyDrag::Track => {
        let (field_of_view, ..) = self.description.get_lens();
        // What a pixel spans at the pivot's distance, so the pivot follows the pointer.
        let per_pixel: f32 = 2.0 * self.focus * (field_of_view.to_radians() / 2.0).tan() / self.height;
        let rotation: Quat = self.get_rotation();

        self.position += rotation * Vec3::X * (-dx * per_pixel) + rotation * Vec3::Y * (dy * per_pixel);
      }
    }
  }

  /// Moves forward by a wheel's pixels turned towards the person, back by those turned away, keeping the pivot where it
  /// stands until the camera reaches it.
  fn wheel(&mut self, pixels: f32) {
    let travel: f32 = -pixels * self.get_speed() * WHEEL_TRAVEL;

    self.position += self.get_rotation() * Vec3::NEG_Z * travel;
    self.focus = (self.focus - travel).max(MIN_FOCUS);
  }

  /// Gathers a wheel's pixels turned while flying into whole steps of speed, a part of one waiting for the rest of it
  /// within the same flight.
  fn step_speed(&mut self, pixels: f32) {
    self.wheeled += pixels;

    let steps: f32 = (self.wheeled / SPEED_WHEEL).trunc();

    self.wheeled -= steps * SPEED_WHEEL;
    // Turned towards the person, slower.
    self.speed_steps -= steps as i32;
  }

  fn turn(&mut self, yaw: f32, pitch: f32) {
    self.yaw += yaw;
    self.pitch = (self.pitch + pitch).clamp(-MAX_PITCH, MAX_PITCH);
  }

  /// Metres a second, times the boost while its key is held.
  fn get_speed(&self) -> f32 {
    let WorldCamera::Fly { speed, boost, .. } = self.description else {
      return 0.0;
    };

    speed * if self.held.contains(&FlyKey::Fast) { boost } else { 1.0 }
  }

  fn get_sensitivity(&self) -> f32 {
    let WorldCamera::Fly { sensitivity, .. } = self.description else {
      return 0.0;
    };

    sensitivity
  }

  /// The point Alt drags turn around and move towards.
  fn get_pivot(&self) -> Vec3 {
    self.position + self.get_rotation() * Vec3::NEG_Z * self.focus
  }

  fn get_rotation(&self) -> Quat {
    Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
  }

  /// The camera's turn alone, level with the ground.
  fn get_heading(&self) -> Quat {
    Quat::from_rotation_y(self.yaw)
  }

  fn axis(&self, positive: FlyKey, negative: FlyKey) -> f32 {
    (self.held.contains(&positive) as i32 - self.held.contains(&negative) as i32) as f32
  }
}
