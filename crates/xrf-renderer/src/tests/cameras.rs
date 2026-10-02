use glam::Vec3;

use crate::camera::fly_camera_controller::FlyCameraController;
use crate::camera::orbit_camera_controller::OrbitCameraController;
use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_input_kind::RenderInputKind;

fn fly(position: [f32; 3], target: [f32; 3]) -> RenderCamera {
  RenderCamera::Fly {
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

fn key(kind: RenderInputKind, code: &str) -> RenderInputEvent {
  RenderInputEvent {
    kind,
    code: code.to_string(),
    ..Default::default()
  }
}

fn pointer(kind: RenderInputKind, x: f32, y: f32) -> RenderInputEvent {
  RenderInputEvent {
    kind,
    is_primary: true,
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

  camera.input(&key(RenderInputKind::KeyDown, "KeyW"));
  camera.update(0.1);
  assert_near(camera.get_pose().position, [0.0, 2.0, -1.0]);

  camera.input(&key(RenderInputKind::KeyDown, "ShiftLeft"));
  camera.update(0.1);
  assert_near(camera.get_pose().position, [0.0, 2.0, -5.0]);

  // A long stall moves no further than a quarter second would.
  camera.input(&key(RenderInputKind::KeyUp, "ShiftLeft"));
  camera.update(10.0);
  assert_near(camera.get_pose().position, [0.0, 2.0, -7.5]);
}

#[test]
fn fly_lets_go_of_every_key_on_blur() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key(RenderInputKind::KeyDown, "KeyW"));
  camera.input(&key(RenderInputKind::Blur, ""));
  camera.update(0.1);

  assert_near(camera.get_pose().position, [0.0, 0.0, 0.0]);
  assert!(!camera.is_moving());
}

#[test]
fn fly_turns_by_a_drag() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&pointer(RenderInputKind::PointerDown, 100.0, 100.0));
  // A quarter turn to the right, at a hundredth of a radian a pixel.
  camera.input(&pointer(
    RenderInputKind::PointerMove,
    100.0 + std::f32::consts::FRAC_PI_2 * 100.0,
    100.0,
  ));
  camera.update(0.016);

  assert_near(camera.get_pose().target, [1.0, 0.0, 0.0]);

  camera.input(&pointer(RenderInputKind::PointerUp, 0.0, 0.0));
  assert!(!camera.is_moving());
}

#[test]
fn fly_keeps_its_place_when_described_again_from_the_same_start() {
  let mut camera: FlyCameraController = FlyCameraController::new(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]));

  camera.input(&key(RenderInputKind::KeyDown, "KeyW"));
  camera.update(0.1);

  assert!(!camera.describe(fly([0.0, 0.0, 0.0], [0.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [0.0, 0.0, -1.0]);

  assert!(camera.describe(fly([3.0, 0.0, 0.0], [3.0, 0.0, -1.0])));
  assert_near(camera.get_pose().position, [3.0, 0.0, 0.0]);
}

#[test]
fn orbit_dollies_and_resets() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(RenderCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [0.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });

  camera.command(RenderCameraCommand::Dolly { step: 0.5 });
  assert_near(camera.get_pose().position, [0.0, 0.0, 2.0]);

  camera.command(RenderCameraCommand::Reset);
  assert_near(camera.get_pose().position, [0.0, 0.0, 4.0]);
}

#[test]
fn orbit_turns_around_its_target_at_the_same_distance() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(RenderCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [1.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });

  camera.resize(400.0);
  camera.input(&pointer(RenderInputKind::PointerDown, 0.0, 0.0));
  camera.input(&pointer(RenderInputKind::PointerMove, 100.0, 40.0));

  let pose = camera.get_pose();

  assert_near(pose.target, [1.0, 0.0, 0.0]);
  assert!((Vec3::from_array(pose.position).distance(Vec3::X) - 17.0f32.sqrt()).abs() < 1e-3);
}

#[test]
fn orbit_pans_its_target_with_it() {
  let mut camera: OrbitCameraController = OrbitCameraController::new(RenderCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [0.0, 0.0, 0.0],
    field_of_view: 45.0,
    near: 0.01,
    far: 100.0,
  });
  let mut down: RenderInputEvent = pointer(RenderInputKind::PointerDown, 0.0, 0.0);

  down.button = 2;
  camera.resize(400.0);
  camera.input(&down);
  camera.input(&pointer(RenderInputKind::PointerMove, -50.0, 0.0));

  let pose = camera.get_pose();

  assert!(pose.target[0] > 0.0);
  assert_near(
    (Vec3::from_array(pose.position) - Vec3::from_array(pose.target)).to_array(),
    [0.0, 0.0, 4.0],
  );
}
