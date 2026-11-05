use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use glam::Vec3;

use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::camera::world_camera::WorldCamera;
use crate::contract::camera::world_camera_command::WorldCameraCommand;
use crate::contract::camera::world_drag_end_reason::WorldDragEndReason;
use crate::contract::camera::world_input_buttons::WorldInputButtons;
use crate::contract::camera::world_input_event::WorldInputEvent;
use crate::contract::camera::world_input_modifiers::WorldInputModifiers;

fn fly(position: [f32; 3], target: [f32; 3]) -> WorldCamera {
  WorldCamera::Fly {
    position,
    target,
    field_of_view: 60.0,
    near: 0.1,
    far: 1000.0,
    speed: 10.0,
    boost: 4.0,
    sensitivity: 0.01,
  }
}

fn orbit(position: [f32; 3], target: [f32; 3]) -> WorldCamera {
  WorldCamera::Orbit {
    position,
    target,
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  }
}

fn key_down(code: &str) -> WorldInputEvent {
  WorldInputEvent::KeyDown { code: code.to_string() }
}

fn key_up(code: &str) -> WorldInputEvent {
  WorldInputEvent::KeyUp { code: code.to_string() }
}

/// A drag of buttons with no modifier held.
fn drag_start(buttons: WorldInputButtons) -> WorldInputEvent {
  WorldInputEvent::DragStart {
    buttons,
    modifiers: WorldInputModifiers::default(),
  }
}

/// A drag of buttons with Alt held.
fn alt_drag_start(buttons: WorldInputButtons) -> WorldInputEvent {
  WorldInputEvent::DragStart {
    buttons,
    modifiers: WorldInputModifiers {
      alt_key: true,
      ..WorldInputModifiers::default()
    },
  }
}

fn drag_move(dx: f32, dy: f32) -> WorldInputEvent {
  WorldInputEvent::DragMove { dx, dy }
}

fn drag_end() -> WorldInputEvent {
  WorldInputEvent::DragEnd {
    reason: WorldDragEndReason::Released,
  }
}

/// A wheel turned by pixels, positive towards the person.
fn wheel(delta_y: f32) -> WorldInputEvent {
  WorldInputEvent::Wheel {
    delta_x: 0.0,
    delta_y,
    delta_mode: 0,
  }
}

fn assert_near(actual: [f32; 3], expected: [f32; 3]) {
  assert!(
    Vec3::from_array(actual).distance(Vec3::from_array(expected)) < 1e-3,
    "{actual:?} is not {expected:?}"
  );
}

#[test]
fn fly_starts_looking_at_its_target() {
  let camera: FlyCameraController = FlyCameraController::new(fly([0.0, 2.0, 0.0], [5.0, 2.0, 0.0]));

  assert_near(camera.get_pose().target, [1.0, 2.0, 0.0]);
}

#[test]
fn fly_flies_where_it_faces_by_the_keys_while_the_secondary_button_is_held() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 2.0, 0.0], [0.0, 2.0, -1.0]));

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.input(&key_down("KeyW"));
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [0.0, 2.0, -1.0]);

  camera.input(&key_down("ShiftLeft"));
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [0.0, 2.0, -5.0]);

  // A long stall moves no further than a quarter second would.
  camera.input(&key_up("ShiftLeft"));
  camera.update(Duration::from_secs(10));
  assert_near(camera.get_pose().position, [0.0, 2.0, -7.5]);
}

#[test]
fn fly_keys_move_nothing_until_the_secondary_button_goes_down() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key_down("KeyW"));
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [0.0, 0.0, 0.0]);

  // The key held first flies as soon as the button joins it.
  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [0.0, 0.0, -1.0]);

  camera.input(&drag_end());
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [0.0, 0.0, -1.0]);
}

#[test]
fn fly_rises_straight_up_however_it_is_pitched() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, -1.0, -1.0]));

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.input(&key_down("KeyE"));
  camera.update(Duration::from_millis(100));

  assert_near(camera.get_pose().position, [0.0, 1.0, 0.0]);
}

#[test]
fn fly_lets_go_of_every_key_and_drag_on_release_all() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key_down("KeyW"));
  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.input(&WorldInputEvent::ReleaseAll);
  camera.update(Duration::from_millis(100));

  assert_near(camera.get_pose().position, [0.0, 0.0, 0.0]);
  assert!(!camera.is_moving());
}

#[test]
fn fly_looks_around_by_a_secondary_drag() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  // A quarter turn to the right in two moves, at a hundredth of a radian a pixel.
  camera.input(&drag_move(FRAC_PI_2 * 50.0, 0.0));
  camera.input(&drag_move(FRAC_PI_2 * 50.0, 0.0));
  camera.update(Duration::from_millis(16));

  assert_near(camera.get_pose().target, [1.0, 0.0, 0.0]);
  assert_near(camera.get_pose().position, [0.0, 0.0, 0.0]);

  camera.input(&drag_end());
  assert!(!camera.is_moving());
}

#[test]
fn fly_walks_along_the_ground_and_turns_by_a_main_drag() {
  // Looking down at the ground ahead, as a level's start does.
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 10.0, 0.0], [0.0, 0.0, -10.0]));

  camera.input(&drag_start(WorldInputButtons::MAIN));
  // Pushed away a hundred pixels, it walks a second's flight forward and keeps its height.
  camera.input(&drag_move(0.0, -100.0));
  assert_near(camera.get_pose().position, [0.0, 10.0, -10.0]);

  // Across, it turns a quarter to the right without moving.
  camera.input(&drag_move(FRAC_PI_2 * 100.0, 0.0));
  assert_near(camera.get_pose().position, [0.0, 10.0, -10.0]);

  camera.input(&drag_move(0.0, -100.0));
  assert_near(camera.get_pose().position, [10.0, 10.0, -10.0]);
}

#[test]
fn fly_pans_sideways_and_up_by_both_buttons_or_the_auxiliary() {
  for buttons in [WorldInputButtons::MAIN_AND_SECONDARY, WorldInputButtons::AUXILIARY] {
    let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, -1.0, -1.0]));

    camera.input(&drag_start(buttons));
    camera.input(&drag_move(100.0, -50.0));

    let pose = camera.get_pose();

    assert_near(pose.position, [10.0, 5.0, 0.0]);
    assert_near(pose.target, [10.0, 5.0 - 0.5f32.sqrt(), -(0.5f32.sqrt())]);
  }
}

#[test]
fn fly_drag_follows_its_buttons_as_they_change() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key_down("KeyW"));
  camera.input(&drag_start(WorldInputButtons::MAIN));
  camera.input(&WorldInputEvent::DragButtons {
    buttons: WorldInputButtons::MAIN_AND_SECONDARY,
  });
  camera.input(&drag_move(100.0, 0.0));
  assert_near(camera.get_pose().position, [10.0, 0.0, 0.0]);

  // The main button let go, the secondary left alone flies by the key held.
  camera.input(&WorldInputEvent::DragButtons {
    buttons: WorldInputButtons::SECONDARY,
  });
  camera.update(Duration::from_millis(100));
  assert_near(camera.get_pose().position, [10.0, 0.0, -1.0]);
}

#[test]
fn fly_wheel_moves_forward_unless_the_secondary_button_flies() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  // A notch away from the person goes a quarter second's flight forward.
  camera.input(&wheel(-100.0));
  assert_near(camera.get_pose().position, [0.0, 0.0, -2.5]);

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.input(&wheel(-100.0));
  assert_near(camera.get_pose().position, [0.0, 0.0, -2.5]);
}

#[test]
fn fly_wheel_asks_for_speed_in_whole_steps_while_the_secondary_button_flies() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&wheel(-100.0));
  assert_eq!(camera.take_speed_steps(), 0);

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  // Two notches away, then a third of one back that waits for the rest of it.
  camera.input(&wheel(-100.0));
  camera.input(&wheel(-100.0));
  camera.input(&WorldInputEvent::Wheel {
    delta_x: 0.0,
    delta_y: 2.0,
    delta_mode: 1,
  });
  assert_eq!(camera.take_speed_steps(), 2);
  assert_eq!(camera.take_speed_steps(), 0);

  camera.input(&wheel(70.0));
  assert_eq!(camera.take_speed_steps(), -1);
}

#[test]
fn fly_orbits_the_target_it_was_described_with_by_an_alt_main_drag() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.input(&alt_drag_start(WorldInputButtons::MAIN));
  camera.input(&drag_move(FRAC_PI_2 * 100.0, 0.0));

  let pose = camera.get_pose();

  assert_near(pose.position, [-4.0, 0.0, 0.0]);
  assert_near(pose.target, [-3.0, 0.0, 0.0]);
}

#[test]
fn fly_dollies_to_its_pivot_by_an_alt_secondary_drag_and_stops_short_of_it() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.input(&alt_drag_start(WorldInputButtons::SECONDARY));
  camera.input(&drag_move(100.0, 0.0));
  assert_near(camera.get_pose().position, [0.0, 0.0, 4.0 * 0.995f32.powi(100)]);

  camera.input(&drag_move(10_000.0, 0.0));
  assert_near(camera.get_pose().position, [0.0, 0.0, 0.25]);
}

#[test]
fn fly_wheel_closes_on_the_pivot_it_then_orbits() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.input(&wheel(-100.0));
  camera.input(&alt_drag_start(WorldInputButtons::MAIN));
  camera.input(&drag_move(FRAC_PI_2 * 100.0, 0.0));

  assert_near(camera.get_pose().position, [-1.5, 0.0, 0.0]);
}

#[test]
fn fly_tracks_its_pivot_under_the_pointer_by_an_alt_auxiliary_drag() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.resize(400.0);
  camera.input(&alt_drag_start(WorldInputButtons::AUXILIARY));
  // At four metres, sixty degrees of view span 8·tan 30° across the viewport's 400 pixels.
  camera.input(&drag_move(-100.0, 0.0));

  let shift: f32 = 8.0 * 30.0f32.to_radians().tan() / 4.0;

  assert_near(camera.get_pose().position, [shift, 0.0, 4.0]);

  // The pivot went with it.
  camera.input(&alt_drag_start(WorldInputButtons::MAIN));
  camera.input(&drag_move(FRAC_PI_2 * 100.0, 0.0));
  assert_near(camera.get_pose().position, [shift - 4.0, 0.0, 0.0]);
}

#[test]
fn fly_keeps_its_place_when_described_again_from_the_same_start() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&drag_start(WorldInputButtons::SECONDARY));
  camera.input(&key_down("KeyW"));
  camera.update(Duration::from_millis(100));

  assert!(!camera.describe(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [0.0, 0.0, -1.0]);

  assert!(camera.describe(fly([3.0, 0.0, 0.0], [3.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [3.0, 0.0, 0.0]);
}

#[test]
fn orbit_dollies_and_resets() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(orbit([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.command(WorldCameraCommand::Dolly { step: 0.5 });
  assert_near(camera.get_pose().position, [0.0, 0.0, 2.0]);

  camera.command(WorldCameraCommand::Reset);
  assert_near(camera.get_pose().position, [0.0, 0.0, 4.0]);
}

#[test]
fn orbit_turns_around_its_target_at_the_same_distance() {
  for start in [
    drag_start(WorldInputButtons::MAIN),
    alt_drag_start(WorldInputButtons::MAIN),
  ] {
    let mut camera: OrbitCameraController = OrbitCameraController::new(orbit([0.0, 0.0, 4.0], [1.0, 0.0, 0.0]));

    camera.resize(400.0);
    camera.input(&start);
    camera.input(&drag_move(100.0, 40.0));

    let pose = camera.get_pose();

    assert_near(pose.target, [1.0, 0.0, 0.0]);
    assert!((Vec3::from_array(pose.position).distance(Vec3::X) - 17.0f32.sqrt()).abs() < 1e-3);
  }
}

#[test]
fn orbit_pans_its_target_with_it() {
  for start in [
    drag_start(WorldInputButtons::SECONDARY),
    alt_drag_start(WorldInputButtons::AUXILIARY),
  ] {
    let mut camera: OrbitCameraController = OrbitCameraController::new(orbit([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

    camera.resize(400.0);
    camera.input(&start);
    camera.input(&drag_move(-50.0, 0.0));

    let pose = camera.get_pose();

    assert!(pose.target[0] > 0.0);
    assert_near(
      (Vec3::from_array(pose.position) - Vec3::from_array(pose.target)).to_array(),
      [0.0, 0.0, 4.0],
    );
  }
}

#[test]
fn orbit_dollies_by_an_alt_secondary_drag() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(orbit([0.0, 0.0, 4.0], [0.0, 0.0, 0.0]));

  camera.input(&alt_drag_start(WorldInputButtons::SECONDARY));
  camera.input(&drag_move(0.0, -100.0));

  assert_near(camera.get_pose().position, [0.0, 0.0, 4.0 * 0.995f32.powi(100)]);
}
