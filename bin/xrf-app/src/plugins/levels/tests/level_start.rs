use std::f32::consts::FRAC_PI_2;

use xrf_math::Vector3d;
use xrf_spawn::SpawnLevelArrival;
use xrf_visual::{VisualBounds, VisualBox, VisualSphere};

use crate::plugins::levels::start::{ACTOR_EYE_HEIGHT, resolve_level_start};
use crate::plugins::levels::state::{LevelSpawn, LevelStart, LevelStartOrigin};

/// A level 200 m across and 100 m along, centred on the origin, in renderer space.
fn new_bounds() -> VisualBounds {
  VisualBounds {
    bounding_box: VisualBox {
      max: Vector3d::new(100.0, 10.0, 50.0),
      min: Vector3d::new(-100.0, -10.0, -50.0),
    },
    bounding_sphere: VisualSphere {
      center: Vector3d::new(0.0, 0.0, 0.0),
      radius: 112.0,
    },
  }
}

fn new_arrival(changer: &str, x: f32, z: f32, heading: f32) -> SpawnLevelArrival {
  SpawnLevelArrival {
    angles: Vector3d::new(0.0, heading, 0.0),
    changer: String::from(changer),
    position: Vector3d::new(x, 2.0, z),
  }
}

fn new_spawn(arrivals: Vec<SpawnLevelArrival>) -> LevelSpawn {
  LevelSpawn {
    arrivals,
    objects: Vec::new(),
  }
}

#[test]
fn opens_at_the_arrival_nearest_the_level_centre() {
  let spawn: LevelSpawn = new_spawn(vec![
    new_arrival("far", 90.0, 40.0, 0.0),
    new_arrival("near", -10.0, 5.0, 0.0),
  ]);
  let start: LevelStart = resolve_level_start(Some(&spawn), Some(&new_bounds()), |_, _| None).expect("a start");

  assert_eq!(start.origin, LevelStartOrigin::Arrival);
  // The engine's z runs the other way, and the eye stands above the ground.
  assert_eq!(start.position, Vector3d::new(-10.0, 2.0 + ACTOR_EYE_HEIGHT, -5.0));
}

// `setHP` of the heading: nought faces the engine's +z, which is the renderer's -z; a quarter turn faces -x. Checked
// against Anomaly's spawn, where 110 of 121 arrivals face their level's centre this way.
#[test]
fn faces_the_heading_the_changer_turns_the_actor_to() {
  let ahead: LevelStart = resolve_level_start(
    Some(&new_spawn(vec![new_arrival("ahead", 0.0, 0.0, 0.0)])),
    Some(&new_bounds()),
    |_, _| None,
  )
  .expect("a start");
  let turned: LevelStart = resolve_level_start(
    Some(&new_spawn(vec![new_arrival("turned", 0.0, 0.0, FRAC_PI_2)])),
    Some(&new_bounds()),
    |_, _| None,
  )
  .expect("a start");

  assert!((ahead.direction.z + 1.0).abs() < 1e-6 && ahead.direction.x.abs() < 1e-6);
  assert!((turned.direction.x + 1.0).abs() < 1e-6 && turned.direction.z.abs() < 1e-6);
}

#[test]
fn stands_on_the_ground_nearest_the_centre_where_no_one_arrives() {
  let start: LevelStart = resolve_level_start(Some(&new_spawn(Vec::new())), Some(&new_bounds()), |x, z| {
    assert_eq!((x, z), (0.0, 0.0), "asked about the centre, in the engine's space");

    Some(Vector3d::new(3.0, -1.0, 4.0))
  })
  .expect("a start");

  assert_eq!(start.origin, LevelStartOrigin::Ground);
  assert_eq!(start.position, Vector3d::new(3.0, -1.0 + ACTOR_EYE_HEIGHT, -4.0));
  assert_eq!(start.direction, Vector3d::new(1.0, 0.0, 0.0), "down the longer side");
}

#[test]
fn leaves_a_level_with_nowhere_to_open_to_the_viewer() {
  assert!(resolve_level_start(None, Some(&new_bounds()), |_, _| None).is_none());
  assert!(
    resolve_level_start(Some(&new_spawn(vec![new_arrival("a", 0.0, 0.0, 0.0)])), None, |_, _| {
      None
    })
    .is_none()
  );
}
