use std::time::Duration;

use xrf_game_loop::{LoopSystem, LoopTime};
use xrf_game_objects::ObjectRecord;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_ltx::Ltx;
use xrf_math::Vector3d;
use xrf_math::glam::Vec3;
use xrf_physics::{PhysicsLevel, PhysicsWorld};

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input::MovementInput;
use crate::input::movement_input_kind::MovementInputKind;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::movement_mode::MovementMode;
use crate::mode::movement_modes::MovementModes;
use crate::mode::walk_mode::WalkMode;
use crate::mode::walk_pace::WalkPace;
use crate::mode::walk_stance::WalkStance;
use crate::movement::Movement;
use crate::movement_command::MovementCommand;
use crate::movement_context::MovementContext;
use crate::tests::fixtures::player;

/// A world holding a floor at `y = 0`, 40 m across, its two faces wound as X-Ray winds them: clockwise from above.
fn floor() -> PhysicsWorld {
  let corner = |x: f32, z: f32| Vector3d { x, y: 0.0, z };
  let face = |vertices: [u32; 3]| LevelCformFace {
    vertices,
    material: 0,
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    sector: 0,
  };
  let geometry: LevelCformGeometry = LevelCformGeometry::new(
    vec![
      corner(-20.0, -20.0),
      corner(-20.0, 20.0),
      corner(20.0, 20.0),
      corner(20.0, -20.0),
    ],
    vec![face([0, 2, 1]), face([0, 3, 2])],
  )
  .expect("a floor");

  let materials: GameMtlFile = GameMtlFile::new(Vec::new(), Vec::new());

  PhysicsWorld::with_level(&PhysicsLevel::of_cform(&geometry, &materials).expect("the floor")).expect("a world")
}

fn press(action: MovementAction, is_pressed: bool) -> MovementCommand {
  MovementCommand::Input(MovementInput {
    at: Duration::ZERO,
    kind: MovementInputKind::Action { action, is_pressed },
  })
}

/// Steps `movement` `ticks` ticks, answering the player's record as it ends.
fn step(movement: &mut Movement, physics: &mut PhysicsWorld, ticks: usize) -> ObjectRecord {
  let time: LoopTime = LoopTime::new(Duration::from_millis(10));
  let mut player: ObjectRecord = player();

  for _ in 0..ticks {
    movement.step(
      &time,
      MovementContext {
        physics: &mut *physics,
        player: &mut player,
      },
    );
  }

  player
}

/// The standing eye's height over the feet.
fn eye() -> f32 {
  WalkMode::default().get_eye_height(WalkStance::Standing)
}

/// Walking from an eye over the floor, looking along `+z`.
fn walking() -> Movement {
  walking_as(WalkMode::default())
}

/// Walking as `walk` sets, from an eye over the floor, looking along `+z`.
fn walking_as(walk: WalkMode) -> Movement {
  Movement::new(
    MovementCamera::looking_at(Vec3::new(0.0, eye(), 0.0), Vec3::new(0.0, eye(), 1.0)),
    MovementMode::Walk,
    MovementModes {
      walk,
      ..MovementModes::default()
    },
  )
}

/// Metres a second along `+z` the actor keeps once up to speed with `actions` held.
fn measure_speed(actions: &[MovementAction]) -> f32 {
  measure_speed_of(walking(), actions)
}

/// Metres a second along `+z` `movement` keeps once up to speed with `actions` held.
fn measure_speed_of(mut movement: Movement, actions: &[MovementAction]) -> f32 {
  let mut physics: PhysicsWorld = floor();

  for action in actions {
    movement.apply(press(*action, true));
  }

  step(&mut movement, &mut physics, 100);

  let from: f32 = movement.get_camera().position.z;

  step(&mut movement, &mut physics, 100);

  movement.get_camera().position.z - from
}

#[test]
fn runs_walks_and_sprints_at_xrays_speeds() {
  // A tenth of `walk_accel` 11.5 times the pace's factors.
  let run: f32 = measure_speed(&[MovementAction::Forward]);
  let walk: f32 = measure_speed(&[MovementAction::Forward, MovementAction::Accel]);
  let sprint: f32 = measure_speed(&[MovementAction::Sprint, MovementAction::Forward]);
  let back: f32 = measure_speed(&[MovementAction::Back]);

  assert!((run - 2.77).abs() < 0.02, "a run of {run} m/s");
  assert!((walk - 1.15).abs() < 0.02, "a walk of {walk} m/s");
  assert!((sprint - 5.76).abs() < 0.05, "a sprint of {sprint} m/s");
  assert!((back + 1.53).abs() < 0.02, "a run back of {back} m/s");
}

#[test]
fn crouches_at_xrays_crouch_factor_and_low_with_accel_held() {
  // `crouch_coef` 0.45 of a run, and of a walk crouched low.
  let crouch: f32 = measure_speed(&[MovementAction::Down, MovementAction::Forward]);
  let low: f32 = measure_speed(&[MovementAction::Down, MovementAction::Accel, MovementAction::Forward]);

  assert!((crouch - 1.247).abs() < 0.02, "a crouch of {crouch} m/s");
  assert!((low - 0.518).abs() < 0.02, "a low crouch of {low} m/s");
}

#[test]
fn lowers_the_eye_to_the_crouched_box_and_raises_it_again() {
  let mode: WalkMode = WalkMode::default();
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  movement.apply(press(MovementAction::Down, true));
  step(&mut movement, &mut physics, 100);

  assert_eq!(movement.get_walk().stance, WalkStance::Crouching);
  assert!((movement.get_camera().position.y - mode.get_eye_height(WalkStance::Crouching)).abs() < 0.01);

  movement.apply(press(MovementAction::Down, false));
  step(&mut movement, &mut physics, 100);

  assert_eq!(movement.get_walk().stance, WalkStance::Standing);
  assert!((movement.get_camera().position.y - eye()).abs() < 0.01);
}

#[test]
fn reports_the_ground_and_pace_it_walks_at() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  movement.apply(press(MovementAction::Forward, true));
  step(&mut movement, &mut physics, 100);

  let footing: MovementFooting = movement.get_footing().expect("walking");

  assert_eq!(footing.ground_material, Some(0));
  assert_eq!(footing.pace, WalkPace::Run);
  assert!((footing.speed - 2.77).abs() < 0.02);

  movement.apply(MovementCommand::ToggleMode);
  step(&mut movement, &mut physics, 1);

  assert_eq!(movement.get_footing(), None);
}

#[test]
fn walks_along_the_ground_however_it_looks_with_the_eye_over_the_feet() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  // Looking down at the floor walks no slower along it.
  movement.apply(MovementCommand::Input(MovementInput {
    at: Duration::ZERO,
    kind: MovementInputKind::Look { dx: 0.0, dy: 300.0 },
  }));
  movement.apply(press(MovementAction::Forward, true));
  step(&mut movement, &mut physics, 200);

  let position: Vec3 = movement.get_camera().position;

  assert!(position.z > 5.0, "{position}");
  assert!((position.y - eye()).abs() < 0.05, "{position}");
  assert!(position.x.abs() < 0.01, "{position}");
}

#[test]
fn stopping_ends_a_sprint() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  movement.apply(press(MovementAction::Sprint, true));
  movement.apply(press(MovementAction::Forward, true));
  step(&mut movement, &mut physics, 10);
  assert!(movement.get_input().is_sprinting());

  movement.apply(press(MovementAction::Forward, false));
  step(&mut movement, &mut physics, 1);
  assert!(!movement.get_input().is_sprinting());
}

#[test]
fn falls_to_the_floor_from_where_flight_left_it() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = Movement::new(
    MovementCamera::looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, 10.0, 1.0)),
    MovementMode::default(),
    MovementModes::default(),
  );

  movement.apply(MovementCommand::ToggleMode);

  let player: ObjectRecord = step(&mut movement, &mut physics, 300);

  assert!((movement.get_camera().position.y - eye()).abs() < 0.05);
  // The player's record stands where the feet do, on the floor, not at the eye.
  assert!(player.position.y.abs() < 0.05, "{}", player.position);
  assert!(player.position.abs_diff_eq(movement.get_feet(), 1e-5));
}

#[test]
fn jumps_as_high_as_xrays_actor_and_lands_again() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();
  let mut highest: f32 = 0.0;

  step(&mut movement, &mut physics, 10);
  movement.apply(press(MovementAction::Up, true));
  step(&mut movement, &mut physics, 1);
  movement.apply(press(MovementAction::Up, false));

  for _ in 0..200 {
    step(&mut movement, &mut physics, 1);
    highest = highest.max(movement.get_camera().position.y);
  }

  // A jump at 6.85 m/s under X-Ray's gravity of 19.62 m/sÂ² rises vÂ² / 2g: 1.2 m.
  assert!((highest - eye() - 1.196).abs() < 0.06, "the eye rose to {highest}");
  assert!((movement.get_camera().position.y - eye()).abs() < 0.05);
}

#[test]
fn lets_go_of_the_actor_when_it_flies_again() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  step(&mut movement, &mut physics, 1);
  assert!(movement.get_walk().actor.is_some());

  movement.apply(MovementCommand::ToggleMode);
  step(&mut movement, &mut physics, 1);
  assert!(movement.get_walk().actor.is_none());
}

#[test]
fn sprints_sideways_but_not_backwards_alone_and_not_walking() {
  let mut physics: PhysicsWorld = floor();
  let mut movement: Movement = walking();

  movement.apply(press(MovementAction::Sprint, true));
  movement.apply(press(MovementAction::Forward, true));
  step(&mut movement, &mut physics, 10);

  // Sideways alone keeps the toggle, as X-Ray's does, though it adds no speed until forward again.
  movement.apply(press(MovementAction::Forward, false));
  movement.apply(press(MovementAction::Right, true));
  step(&mut movement, &mut physics, 10);
  assert!(movement.get_input().is_sprinting());
  assert_eq!(movement.get_footing().map(|footing| footing.pace), Some(WalkPace::Run));

  // Back alone drops it.
  movement.apply(press(MovementAction::Right, false));
  movement.apply(press(MovementAction::Back, true));
  step(&mut movement, &mut physics, 1);
  assert!(!movement.get_input().is_sprinting());

  // So does walking.
  movement.apply(press(MovementAction::Back, false));
  movement.apply(press(MovementAction::Sprint, true));
  movement.apply(press(MovementAction::Forward, true));
  step(&mut movement, &mut physics, 1);
  movement.apply(press(MovementAction::Accel, true));
  step(&mut movement, &mut physics, 1);
  assert!(!movement.get_input().is_sprinting());
}

#[test]
fn strafes_at_the_strafe_factors_standing() {
  let side = |run_strafe: f32| -> f32 {
    let mut physics: PhysicsWorld = floor();
    let mut movement: Movement = walking_as(WalkMode {
      run_strafe,
      ..WalkMode::default()
    });

    movement.apply(press(MovementAction::Right, true));
    step(&mut movement, &mut physics, 100);

    let from: f32 = movement.get_camera().position.x;

    step(&mut movement, &mut physics, 100);

    (movement.get_camera().position.x - from).abs()
  };

  assert!((side(1.0) - 2.77).abs() < 0.02, "a strafe of {}", side(1.0));
  assert!((side(0.5) - 1.385).abs() < 0.02, "a half strafe of {}", side(0.5));
}

#[test]
fn reads_the_trees_actor_and_keeps_it_through_noclip() {
  let ltx: Ltx = Ltx::read_from_str(
    "[actor]\nwalk_accel = 19\nrun_coef = 2.0\nrun_back_coef = 1.8\nwalk_back_coef = 0.5\nsprint_koef = 2.5\n\
     crouch_coef = 0.30\nair_control_param = 0.1\njump_speed = 6.\ncamera_height_factor = 0.92\nph_mass = 94\n\
     ph_box0_size = 0.30, 0.90, 0.30\nph_box1_size = 0.35, 0.67, 0.30\nph_box2_size = 0.35, 0.22, 0.35\n",
  )
  .expect("Call of Pripyat's [actor]");
  let walk: WalkMode = WalkMode::read_ltx(&ltx);

  assert!((walk.get_speed(walk.run) - 3.8).abs() < 1e-5);
  assert!((walk.crouch_height - 1.34).abs() < 1e-5);
  assert!((walk.actor.height - 1.8).abs() < 1e-5);
  assert!((walk.walk_strafe - 1.0).abs() < 1e-5, "1 where the configs set none");

  // Flying and walking again keeps the tree's walk, not the default.
  let mut movement: Movement = walking_as(walk);

  movement.apply(MovementCommand::ToggleMode);
  movement.apply(MovementCommand::ToggleMode);

  let run: f32 = measure_speed_of(movement, &[MovementAction::Forward]);

  assert!((run - 3.8).abs() < 0.03, "a run of {run} m/s");
}
