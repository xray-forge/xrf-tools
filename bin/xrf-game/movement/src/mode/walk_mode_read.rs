use xrf_game_config::ConfigReader;
use xrf_math::glam::Vec3;
use xrf_physics::PhysicsActorSettings;

use crate::mode::walk_mode::WalkMode;

impl WalkMode {
  /// The configs' section the actor's movement is read from.
  pub const ACTOR_SECTION: &'static str = "actor";

  /// Walking as the tree's actor walks, read from its `[actor]` (`CActor::Load`, `Actor.cpp`): its acceleration and
  /// the pace factors, air control, jump and eye, and its boxes' heights and mass. X-Ray reads each unconditioned but
  /// the strafe factors, so one missing or not of its kind is an issue and keeps Anomaly's; the strafe factors are 1
  /// where they are not set, as neither Call of Pripyat nor Anomaly sets them.
  pub fn read(section: &mut ConfigReader<'_, '_>) -> Self {
    let defaults: Self = Self::default();
    let mode: Self = Self {
      walk_accel: section.require_or("walk_accel", defaults.walk_accel),
      run: section.require_or("run_coef", defaults.run),
      run_back: section.require_or("run_back_coef", defaults.run_back),
      walk_back: section.require_or("walk_back_coef", defaults.walk_back),
      sprint: section.require_or("sprint_koef", defaults.sprint),
      crouch: section.require_or("crouch_coef", defaults.crouch),
      walk_strafe: section.read_or("walk_strafe_coef", 1.0),
      run_strafe: section.read_or("run_strafe_coef", 1.0),
      air_control: section.require_or("air_control_param", defaults.air_control),
      jump_speed: section.require_or("jump_speed", defaults.jump_speed),
      crouch_height: Self::read_box_height(section, "ph_box1_size").unwrap_or(defaults.crouch_height),
      low_height: Self::read_box_height(section, "ph_box2_size").unwrap_or(defaults.low_height),
      eye_factor: section.require_or("camera_height_factor", defaults.eye_factor),
      actor: PhysicsActorSettings {
        height: Self::read_box_height(section, "ph_box0_size").unwrap_or(defaults.actor.height),
        mass: section.require_or("ph_mass", defaults.actor.mass),
        ..defaults.actor
      },
      ..defaults
    };

    log::info!(
      "Walking as [{}]: runs at {:.2} m/s, walks at {:.2}, sprints at {:.2}, crouches at {:.2}, jumps at {} m/s",
      Self::ACTOR_SECTION,
      mode.get_speed(mode.run),
      mode.get_speed(1.0),
      mode.get_speed(mode.run * mode.sprint),
      mode.get_speed(mode.run * mode.crouch),
      mode.jump_speed
    );

    mode
  }

  /// The height of the box `key` sizes: twice its half-extent upwards, `0.30, 0.90, 0.30` 1.8 m high; none, said as an
  /// issue, for a box of no height.
  fn read_box_height(section: &mut ConfigReader<'_, '_>, key: &str) -> Option<f32> {
    let half: Vec3 = section.require(key)?;

    if half.y <= 0.0 {
      section.push_key(key, "sizes a box of no height");

      return None;
    }

    Some(half.y * 2.0)
  }
}
