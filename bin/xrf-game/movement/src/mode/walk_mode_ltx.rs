use xrf_ltx::{Ltx, Section};
use xrf_physics::PhysicsActorSettings;

use crate::mode::walk_mode::WalkMode;

impl WalkMode {
  /// The configs' section the actor's movement is read from.
  pub const ACTOR_SECTION: &'static str = "actor";

  /// Walking as the tree's actor walks, read from `ltx`'s `[actor]` (`CActor::Load`, `Actor.cpp`): its acceleration
  /// and the pace factors, air control, jump and eye, and its boxes' heights and mass. A key the configs lack or that
  /// does not read keeps the default, and is logged; where there is no `[actor]` it all does.
  pub fn read_ltx(ltx: &Ltx) -> Self {
    let defaults: Self = Self::default();
    let Some(section) = ltx.section(Self::ACTOR_SECTION) else {
      log::warn!(
        "The configs hold no [{}]; walking moves as Anomaly's actor",
        Self::ACTOR_SECTION
      );

      return defaults;
    };
    let mut kept: Vec<&str> = Vec::new();
    let mut read = |key: &'static str, default: f32| -> f32 {
      Self::read_number(section, key).unwrap_or_else(|| {
        kept.push(key);

        default
      })
    };
    let walk_accel: f32 = read("walk_accel", defaults.walk_accel);
    let run: f32 = read("run_coef", defaults.run);
    let run_back: f32 = read("run_back_coef", defaults.run_back);
    let walk_back: f32 = read("walk_back_coef", defaults.walk_back);
    let sprint: f32 = read("sprint_koef", defaults.sprint);
    let crouch: f32 = read("crouch_coef", defaults.crouch);
    let air_control: f32 = read("air_control_param", defaults.air_control);
    let jump_speed: f32 = read("jump_speed", defaults.jump_speed);
    let eye_factor: f32 = read("camera_height_factor", defaults.eye_factor);
    let mass: f32 = read("ph_mass", defaults.actor.mass);
    // X-Ray reads these two only where they are set, 1 otherwise; neither Call of Pripyat nor Anomaly sets them.
    let walk_strafe: f32 = Self::read_number(section, "walk_strafe_coef").unwrap_or(1.0);
    let run_strafe: f32 = Self::read_number(section, "run_strafe_coef").unwrap_or(1.0);
    let height: f32 = Self::read_box_height(section, "ph_box0_size").unwrap_or_else(|| {
      kept.push("ph_box0_size");

      defaults.actor.height
    });
    let crouch_height: f32 = Self::read_box_height(section, "ph_box1_size").unwrap_or_else(|| {
      kept.push("ph_box1_size");

      defaults.crouch_height
    });
    let low_height: f32 = Self::read_box_height(section, "ph_box2_size").unwrap_or_else(|| {
      kept.push("ph_box2_size");

      defaults.low_height
    });

    if !kept.is_empty() {
      log::warn!(
        "[{}] lacks or cannot read {}; walking keeps Anomaly's",
        Self::ACTOR_SECTION,
        kept.join(", ")
      );
    }

    let mode: Self = Self {
      walk_accel,
      run,
      run_back,
      walk_back,
      sprint,
      crouch,
      walk_strafe,
      run_strafe,
      air_control,
      jump_speed,
      crouch_height,
      low_height,
      eye_factor,
      actor: PhysicsActorSettings {
        height,
        mass,
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

  /// The number `key` holds, none where it is missing or no number.
  fn read_number(section: &Section, key: &str) -> Option<f32> {
    section.read_optional_field::<f32>(key).ok().flatten()
  }

  /// The height of the box `key` sizes: twice the second of its half-extents, `0.30, 0.90, 0.30` 1.8 m high.
  fn read_box_height(section: &Section, key: &str) -> Option<f32> {
    let half: f32 = section.get(key)?.split(',').nth(1)?.trim().parse::<f32>().ok()?;

    (half > 0.0).then_some(half * 2.0)
  }
}
