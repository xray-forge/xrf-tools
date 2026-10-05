use glam::Vec3;

use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::world_camera::WorldCamera;
use crate::contract::world_camera_command::WorldCameraCommand;
use crate::contract::world_input_event::WorldInputEvent;
use crate::contract::world_input_kind::WorldInputKind;

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

fn key(kind: WorldInputKind, code: &str) -> WorldInputEvent {
  WorldInputEvent {
    kind,
    code: code.to_string(),
    ..Default::default()
  }
}

/// A gesture of the main pointer, its main button held through a press and the moves after it.
fn pointer(kind: WorldInputKind, x: f32, y: f32) -> WorldInputEvent {
  WorldInputEvent {
    kind,
    is_primary: true,
    buttons: u32::from(matches!(
      kind,
      WorldInputKind::PointerDown | WorldInputKind::PointerMove
    )),
    x,
    y,
    ..Default::default()
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
fn fly_walks_where_it_faces_and_faster_boosted() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 2.0, 0.0], [0.0, 2.0, -1.0]));

  camera.input(&key(WorldInputKind::KeyDown, "KeyW"));
  camera.update(0.1);
  assert_near(camera.get_pose().position, [0.0, 2.0, -1.0]);

  camera.input(&key(WorldInputKind::KeyDown, "ShiftLeft"));
  camera.update(0.1);
  assert_near(camera.get_pose().position, [0.0, 2.0, -5.0]);

  // A long stall moves no further than a quarter second would.
  camera.input(&key(WorldInputKind::KeyUp, "ShiftLeft"));
  camera.update(10.0);
  assert_near(camera.get_pose().position, [0.0, 2.0, -7.5]);
}

#[test]
fn fly_lets_go_of_every_key_on_blur() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key(WorldInputKind::KeyDown, "KeyW"));
  camera.input(&key(WorldInputKind::Blur, ""));
  camera.update(0.1);

  assert_near(camera.get_pose().position, [0.0, 0.0, 0.0]);
  assert!(!camera.is_moving());
}

#[test]
fn fly_turns_by_a_drag() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&pointer(WorldInputKind::PointerDown, 100.0, 100.0));
  // A quarter turn to the right, at a hundredth of a radian a pixel.
  camera.input(&pointer(
    WorldInputKind::PointerMove,
    100.0 + std::f32::consts::FRAC_PI_2 * 100.0,
    100.0,
  ));
  camera.update(0.016);

  assert_near(camera.get_pose().target, [1.0, 0.0, 0.0]);

  camera.input(&pointer(WorldInputKind::PointerUp, 0.0, 0.0));
  assert!(!camera.is_moving());
}

// A release lost on its way, to a capture the page dropped, would otherwise leave a plain hover turning the camera.
#[test]
fn fly_ends_a_drag_on_a_move_with_no_button_held() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));
  let mut hover: WorldInputEvent = pointer(WorldInputKind::PointerMove, 300.0, 100.0);

  hover.buttons = 0;
  camera.input(&pointer(WorldInputKind::PointerDown, 100.0, 100.0));
  camera.input(&hover);
  camera.input(&pointer(WorldInputKind::PointerMove, 400.0, 100.0));
  camera.update(0.016);

  assert_near(camera.get_pose().target, [0.0, 0.0, -1.0]);
}

#[test]
fn fly_keeps_its_place_when_described_again_from_the_same_start() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key(WorldInputKind::KeyDown, "KeyW"));
  camera.update(0.1);

  assert!(!camera.describe(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [0.0, 0.0, -1.0]);

  assert!(camera.describe(fly([3.0, 0.0, 0.0], [3.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [3.0, 0.0, 0.0]);
}

#[test]
fn orbit_dollies_and_resets() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(WorldCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [0.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });

  camera.command(WorldCameraCommand::Dolly { step: 0.5 });
  assert_near(camera.get_pose().position, [0.0, 0.0, 2.0]);

  camera.command(WorldCameraCommand::Reset);
  assert_near(camera.get_pose().position, [0.0, 0.0, 4.0]);
}

#[test]
fn orbit_turns_around_its_target_at_the_same_distance() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(WorldCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [1.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });

  camera.resize(400.0);
  camera.input(&pointer(WorldInputKind::PointerDown, 0.0, 0.0));
  camera.input(&pointer(WorldInputKind::PointerMove, 100.0, 40.0));

  let pose = camera.get_pose();

  assert_near(pose.target, [1.0, 0.0, 0.0]);
  assert!((Vec3::from_array(pose.position).distance(Vec3::X) - 17.0f32.sqrt()).abs() < 1e-3);
}

#[test]
fn orbit_pans_its_target_with_it() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(WorldCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [0.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });
  let mut down: WorldInputEvent = pointer(WorldInputKind::PointerDown, 0.0, 0.0);

  down.button = 2;
  camera.resize(400.0);
  camera.input(&down);
  camera.input(&pointer(WorldInputKind::PointerMove, -50.0, 0.0));

  let pose = camera.get_pose();

  assert!(pose.target[0] > 0.0);
  assert_near(
    (Vec3::from_array(pose.position) - Vec3::from_array(pose.target)).to_array(),
    [0.0, 0.0, 4.0],
  );
}
