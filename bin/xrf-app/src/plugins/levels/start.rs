//! Where a level opens: where the game has a player arrive, the ground nearest its middle where it has none.

use xrf_math::Vector3d;
use xrf_spawn::{AlifeObjectInherited, SpawnLevelArrival};
use xrf_visual::VisualBounds;

use crate::plugins::levels::state::{LevelSpawn, LevelStart, LevelStartOrigin};

/// Metres an actor's eye stands above the ground under it.
pub const ACTOR_EYE_HEIGHT: f32 = 1.7;

/// The start of a level, the first the level has of: the arrival nearest its centre, its actor's spawn, and the ground
/// `find_ground` finds nearest the centre, given in the engine's space; none of them leaves it to the viewer.
pub fn resolve_level_start(
  spawn: Option<&LevelSpawn>,
  bounds: Option<&VisualBounds>,
  find_ground: impl FnOnce(f32, f32) -> Option<Vector3d>,
) -> Option<LevelStart> {
  let bounds: &VisualBounds = bounds?;
  let (x, z): (f32, f32) = to_engine_centre(bounds);

  if let Some(spawn) = spawn {
    let nearest: Option<&SpawnLevelArrival> = spawn
      .arrivals
      .iter()
      .min_by(|left, right| to_distance(&left.position, x, z).total_cmp(&to_distance(&right.position, x, z)));

    if let Some(arrival) = nearest {
      return Some(to_start(&arrival.position, arrival.angles.y, LevelStartOrigin::Arrival));
    }

    let actor = spawn
      .objects
      .iter()
      .find(|object| matches!(object.inherited, AlifeObjectInherited::SeActor(_)));

    if let Some(actor) = actor {
      return Some(to_start(&actor.position, actor.direction.y, LevelStartOrigin::Actor));
    }
  }

  find_ground(x, z).map(|ground| LevelStart {
    direction: to_longer_side(bounds),
    origin: LevelStartOrigin::Ground,
    position: to_renderer_eye(&ground),
  })
}

/// An actor's eye at a place, turned to a heading as `CActor::MoveActor` turns it: `setHP` of the heading.
fn to_start(position: &Vector3d, heading: f32, origin: LevelStartOrigin) -> LevelStart {
  LevelStart {
    direction: Vector3d::new(-heading.sin(), 0.0, -heading.cos()),
    origin,
    position: to_renderer_eye(position),
  }
}

/// A place in the engine's space, as an eye above it in the renderer's, whose z runs the other way.
fn to_renderer_eye(position: &Vector3d) -> Vector3d {
  Vector3d::new(position.x, position.y + ACTOR_EYE_HEIGHT, -position.z)
}

/// The middle of an extent in the renderer's space, as the engine's x and z.
fn to_engine_centre(bounds: &VisualBounds) -> (f32, f32) {
  let VisualBounds { bounding_box, .. } = bounds;

  (
    (bounding_box.min.x + bounding_box.max.x) / 2.0,
    -(bounding_box.min.z + bounding_box.max.z) / 2.0,
  )
}

fn to_distance(position: &Vector3d, x: f32, z: f32) -> f32 {
  (position.x - x).powi(2) + (position.z - z).powi(2)
}

/// Down the extent's longer side, so most of the level is in front.
fn to_longer_side(bounds: &VisualBounds) -> Vector3d {
  let across: f32 = bounds.bounding_box.max.x - bounds.bounding_box.min.x;
  let along: f32 = bounds.bounding_box.max.z - bounds.bounding_box.min.z;

  if across >= along {
    Vector3d::new(1.0, 0.0, 0.0)
  } else {
    Vector3d::new(0.0, 0.0, -1.0)
  }
}
