use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use xrf_math::glam::Vec3;

use crate::camera::camera_controller::CameraController;
use crate::contract::camera::world_camera::WorldCamera;
use crate::host::world_fed_camera::WorldFedCamera;
use crate::host::world_fed_clock::WorldFedClock;
use crate::host::world_fed_view::WorldFedView;
use crate::host::world_view_feed::WorldViewFeed;
use crate::weather::weather_player::WeatherPlayer;

/// A host's feed answering whatever view it was last given.
#[derive(Default)]
struct HeldFeed {
  view: Mutex<WorldFedView>,
}

impl HeldFeed {
  fn set(&self, view: WorldFedView) {
    *self.view.lock().unwrap_or_else(PoisonError::into_inner) = view;
  }
}

impl WorldViewFeed for HeldFeed {
  fn read(&self, _: Instant) -> WorldFedView {
    *self.view.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-4), "{actual} is not {expected}");
}

fn clock(hours: f32) -> WorldFedClock {
  WorldFedClock {
    time: hours * 3600.0,
    rate: 10.0,
  }
}

#[test]
fn draws_the_hosts_camera_in_the_renderers_space() {
  let feed: Arc<HeldFeed> = Arc::new(HeldFeed::default());
  let mut camera: CameraController = CameraController::default();

  camera.feed(Arc::clone(&feed) as Arc<dyn WorldViewFeed>);
  feed.set(WorldFedView {
    camera: WorldFedCamera {
      position: Vec3::new(1.0, 2.0, 3.0),
      forward: Vec3::X,
      ..WorldFedCamera::default()
    },
    clock: None,
  });
  camera.update(Instant::now(), Duration::from_millis(16), 600.0);

  let frame = camera.get_frame();

  // The engine's `z` runs the other way.
  assert_near(frame.position, Vec3::new(1.0, 2.0, -3.0));
  // The view looks down its `-z` along the forward given.
  assert_near(frame.view.transform_vector3(Vec3::X), Vec3::NEG_Z);
  assert_near(Vec3::from(camera.get_pose().target), Vec3::new(2.0, 2.0, -3.0));
  assert_eq!(frame.field_of_view, WorldFedCamera::default().field_of_view);
}

#[test]
fn hands_the_camera_back_when_one_is_described() {
  let mut camera: CameraController = CameraController::default();

  camera.feed(Arc::new(HeldFeed::default()));
  assert!(matches!(camera, CameraController::Fed(_)));

  camera.describe(WorldCamera::Orbit {
    position: [0.0, 0.0, 4.0],
    target: [0.0; 3],
    field_of_view: 45.0,
    near: 0.1,
    far: 100.0,
  });
  assert!(matches!(camera, CameraController::Orbit(_)));
  assert_eq!(camera.get_clock(), None);
}

#[test]
fn weather_follows_the_hosts_clock_running_on_or_seeking() {
  let mut player: WeatherPlayer = WeatherPlayer::default();

  // From noon to ten in the morning is most of a day ahead: a seek.
  player.follow_clock(Some(clock(10.0)));
  assert!((player.get_time() - 10.0 * 3600.0).abs() < 1e-2);
  assert_eq!(player.get_clock_rate(), 10.0);

  // Half an hour on is run, across midnight as well.
  player.follow_clock(Some(clock(10.5)));
  assert!((player.get_time() - 10.5 * 3600.0).abs() < 1e-2);

  player.follow_clock(Some(clock(23.9)));
  player.follow_clock(Some(clock(0.2)));
  assert!((player.get_time() - 0.2 * 3600.0).abs() < 1e-2);

  // Let go, it runs its own clock, paused by default.
  player.follow_clock(None);
  assert_eq!(player.get_clock_rate(), 0.0);
}
