//! Where a level opens: where the game has a player arrive or spawns its actor, under the open sky.

use xrf_math::Vector3d;
use xrf_spawn::AlifeObjectInherited;
use xrf_visual::{VisualBounds, convert_vector};

use crate::plugins::levels::state::{LevelSpawn, LevelStart, LevelStartOrigin};

/// Metres an actor's eye stands above the ground under it.
pub const ACTOR_EYE_HEIGHT: f32 = 1.7;

/// The eyes a level may open at, in the engine's space: where each of its arrivals and its actor's spawn stand.
pub fn list_level_start_eyes(spawn: &LevelSpawn) -> Vec<Vector3d> {
  list_candidates(spawn).map(|(eye, _, _)| eye).collect()
}

/// The start of a level: of the eyes `list_level_start_eyes` lists, the one nearest its centre that `open` says is
/// under the open sky; none of them leaves it to the viewer.
pub fn resolve_level_start(
  spawn: Option<&LevelSpawn>,
  bounds: Option<&VisualBounds>,
  open: &[bool],
) -> Option<LevelStart> {
  let (x, z): (f32, f32) = to_engine_centre(bounds?);

  list_candidates(spawn?)
    .zip(open)
    .filter_map(|(candidate, is_open)| is_open.then_some(candidate))
    .min_by(|(left, _, _), (right, _, _)| to_distance(left, x, z).total_cmp(&to_distance(right, x, z)))
    .map(|(eye, heading, origin)| to_start(&eye, heading, origin))
}

/// Each arrival, then the actor's spawn, as an eye, a heading and what it is.
fn list_candidates(spawn: &LevelSpawn) -> impl Iterator<Item = (Vector3d, f32, LevelStartOrigin)> {
  let arrivals = spawn
    .arrivals
    .iter()
    .map(|arrival| (to_eye(&arrival.position), arrival.angles.y, LevelStartOrigin::Arrival));
  let actors = spawn
    .objects
    .iter()
    .filter(|object| matches!(object.inherited, AlifeObjectInherited::SeActor(_)))
    .map(|actor| (to_eye(&actor.position), actor.direction.y, LevelStartOrigin::Actor));

  arrivals.chain(actors)
}

/// An actor's eye above a place, in the engine's space.
fn to_eye(position: &Vector3d) -> Vector3d {
  Vector3d::new(position.x, position.y + ACTOR_EYE_HEIGHT, position.z)
}

/// An eye in the renderer's space, turned to a heading as `CActor::MoveActor` turns it: the forward axis of `setHP` of
/// the heading.
fn to_start(eye: &Vector3d, heading: f32, origin: LevelStartOrigin) -> LevelStart {
  LevelStart {
    direction: convert_vector(&Vector3d::new(-heading.sin(), 0.0, heading.cos())),
    origin,
    position: convert_vector(eye),
  }
}

/// The middle of an extent in the renderer's space, as the engine's x and z: the conversion is its own inverse.
fn to_engine_centre(bounds: &VisualBounds) -> (f32, f32) {
  let VisualBounds { bounding_box, .. } = bounds;
  let centre: Vector3d = convert_vector(&Vector3d::new(
    (bounding_box.min.x + bounding_box.max.x) / 2.0,
    (bounding_box.min.y + bounding_box.max.y) / 2.0,
    (bounding_box.min.z + bounding_box.max.z) / 2.0,
  ));

  (centre.x, centre.z)
}

fn to_distance(position: &Vector3d, x: f32, z: f32) -> f32 {
  (position.x - x).powi(2) + (position.z - z).powi(2)
}
