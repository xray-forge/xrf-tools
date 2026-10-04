use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use xrf_math::Vector3d;
use xrf_spawn::SpawnLevelArrival;
use xrf_visual::{VisualBounds, VisualBox, VisualSphere};

use crate::plugins::levels::start::{ACTOR_EYE_HEIGHT, list_level_start_eyes, resolve_level_start};
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
    releases: HashMap::new(),
  }
}

#[test]
fn opens_at_the_arrival_nearest_the_level_centre() {
  let spawn: LevelSpawn = new_spawn(vec![
    new_arrival("far", 90.0, 40.0, 0.0),
    new_arrival("near", -10.0, 5.0, 0.0),
  ]);
  let start: LevelStart = resolve_level_start(Some(&spawn), Some(&new_bounds()), &[true, true]).expect("a start");

  assert_eq!(start.origin, LevelStartOrigin::Arrival);
  // The engine's z runs the other way, and the eye stands above the ground.
  assert_eq!(start.position, Vector3d::new(-10.0, 2.0 + ACTOR_EYE_HEIGHT, -5.0));
}

// Pripyat's arrival nearest its centre is the one from Lab X8, in a room: a level opens where it can be seen.
#[test]
fn passes_over_an_arrival_with_something_over_it() {
  let spawn: LevelSpawn = new_spawn(vec![
    new_arrival("far", 90.0, 40.0, 0.0),
    new_arrival("indoors", -10.0, 5.0, 0.0),
  ]);

  // Asked of every eye at once, in the engine's space.
  assert_eq!(
    list_level_start_eyes(&spawn),
    [
      Vector3d::new(90.0, 2.0 + ACTOR_EYE_HEIGHT, 40.0),
      Vector3d::new(-10.0, 2.0 + ACTOR_EYE_HEIGHT, 5.0)
    ]
  );

  let start: LevelStart = resolve_level_start(Some(&spawn), Some(&new_bounds()), &[true, false]).expect("a start");

  assert_eq!(start.position, Vector3d::new(90.0, 2.0 + ACTOR_EYE_HEIGHT, -40.0));
}

// `setHP` of the heading: nought faces the engine's +z, which is the renderer's -z; a quarter turn faces -x. Checked
// against Anomaly's spawn, where 110 of 121 arrivals face their level's centre this way.
#[test]
fn faces_the_heading_the_changer_turns_the_actor_to() {
  let ahead: LevelStart = resolve_level_start(
    Some(&new_spawn(vec![new_arrival("ahead", 0.0, 0.0, 0.0)])),
    Some(&new_bounds()),
    &[true],
  )
  .expect("a start");
  let turned: LevelStart = resolve_level_start(
    Some(&new_spawn(vec![new_arrival("turned", 0.0, 0.0, FRAC_PI_2)])),
    Some(&new_bounds()),
    &[true],
  )
  .expect("a start");

  assert!((ahead.direction.z + 1.0).abs() < 1e-6 && ahead.direction.x.abs() < 1e-6);
  assert!((turned.direction.x + 1.0).abs() < 1e-6 && turned.direction.z.abs() < 1e-6);
}

// A viewer frames the whole level instead, which never stands inside anything.
#[test]
fn leaves_a_level_with_nowhere_open_to_the_viewer() {
  let spawn: LevelSpawn = new_spawn(vec![new_arrival("a", 0.0, 0.0, 0.0)]);

  assert!(resolve_level_start(Some(&spawn), Some(&new_bounds()), &[false]).is_none());
  assert!(resolve_level_start(Some(&new_spawn(Vec::new())), Some(&new_bounds()), &[]).is_none());
  assert!(resolve_level_start(None, Some(&new_bounds()), &[true]).is_none());
  assert!(resolve_level_start(Some(&spawn), None, &[true]).is_none());
}
