use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_level::LevelCformFile;
use xrf_math::Vector3d;
use xrf_spawn::AlifeObjectInherited;
use xrf_vfs::{XrayInputRecord, XraySearch};
use xrf_visual::{SectorOutline, VisualBounds, convert_vector};

use crate::level::{LevelSource, SelectedLevel};
use crate::parts::{LevelCollisionPart, LevelSpawn, LevelSpawnPart, LevelStart, LevelStartOrigin};
use crate::report::LevelReport;

/// What a change event calls where a level opens when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_START_PART: &str = "start";

/// Where the level opens: where the game has a player arrive or spawns its actor, under the open sky. Decided over its
/// spawn and what of `level.cform` stands over each eye, so a change to either moves the start alone and not the level.
pub(crate) struct LevelStartPart;

impl SessionPart for LevelStartPart {
  type Value = Option<LevelStart>;

  const NAME: &'static str = LEVEL_START_PART;
}

impl LevelStartPart {
  /// Metres an actor's eye stands above the ground under it.
  pub const ACTOR_EYE_HEIGHT: f32 = 1.7;

  /// The store with `start` kept as the level's start part, decided from `inputs` as the level opened.
  pub fn with(parts: SessionParts, start: Option<LevelStart>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(start, inputs)
  }

  /// Where the open level opens, decided the first time anything asks and kept; `None` leaves it to the viewer.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Option<LevelStart> {
    current.parts.get_or_record::<Self>(search, |search| {
      let spawn = LevelSpawnPart::get(current, search).ok();
      let open: Vec<bool> = spawn.as_deref().map_or_else(Vec::new, |spawn| {
        Self::find_open(&current.source, search, &Self::list_eyes(spawn))
      });

      Self::resolve(
        spawn.as_deref(),
        SectorOutline::merge_bounds(&current.outlines).as_ref(),
        &open,
      )
    })
  }

  /// Whether nothing of the collision form stands over each eye; every one for a level without a readable form, which
  /// covers nothing.
  pub fn find_open(source: &LevelSource, search: &XraySearch, eyes: &[Vector3d]) -> Vec<bool> {
    source
      .read_optional_file(search, LevelCollisionPart::FILE)
      .inspect_err(|error| LevelReport::unreadable_open_sky(source, error))
      .ok()
      .flatten()
      .and_then(|bytes| {
        LevelCformFile::read_open_above_from_bytes::<XRayByteOrder>(bytes, eyes)
          .inspect_err(|error| {
            LevelReport::unreadable_open_sky(source, &source.describe_read_failure(LevelCollisionPart::FILE, error))
          })
          .ok()
      })
      .unwrap_or_else(|| vec![true; eyes.len()])
  }

  /// The eyes a level may open at, in the engine's space: where each of its arrivals and its actor's spawn stand.
  pub fn list_eyes(spawn: &LevelSpawn) -> Vec<Vector3d> {
    Self::list_candidates(spawn).map(|(eye, _, _)| eye).collect()
  }

  /// The start of a level: of the eyes `list_eyes` lists, the one nearest its centre that `open` says is under the open
  /// sky; none of them leaves it to the viewer.
  pub fn resolve(spawn: Option<&LevelSpawn>, bounds: Option<&VisualBounds>, open: &[bool]) -> Option<LevelStart> {
    let (x, z): (f32, f32) = Self::to_engine_centre(bounds?);

    Self::list_candidates(spawn?)
      .zip(open)
      .filter_map(|(candidate, is_open)| is_open.then_some(candidate))
      .min_by(|(left, _, _), (right, _, _)| Self::to_distance(left, x, z).total_cmp(&Self::to_distance(right, x, z)))
      .map(|(eye, heading, origin)| Self::to_start(&eye, heading, origin))
  }

  /// Each arrival, then the actor's spawn, as an eye, a heading and what it is.
  fn list_candidates(spawn: &LevelSpawn) -> impl Iterator<Item = (Vector3d, f32, LevelStartOrigin)> {
    let arrivals = spawn.arrivals.iter().map(|arrival| {
      (
        Self::to_eye(&arrival.position),
        arrival.angles.y,
        LevelStartOrigin::Arrival,
      )
    });
    let actors = spawn
      .list_kept()
      .filter(|object| matches!(object.inherited, AlifeObjectInherited::SeActor(_)))
      .map(|actor| {
        (
          Self::to_eye(&actor.position),
          actor.direction.y,
          LevelStartOrigin::Actor,
        )
      });

    arrivals.chain(actors)
  }

  /// An actor's eye above a place, in the engine's space.
  fn to_eye(position: &Vector3d) -> Vector3d {
    Vector3d::new(position.x, position.y + Self::ACTOR_EYE_HEIGHT, position.z)
  }

  /// An eye in the renderer's space, turned to a heading as `CActor::MoveActor` turns it: the forward axis of `setHP`
  /// of the heading.
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
}
