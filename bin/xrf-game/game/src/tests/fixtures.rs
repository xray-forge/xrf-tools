use xrf_game_alife::{Alife, AlifeData, AlifeHooks, AlifeSettings};
use xrf_game_movement::{Movement, MovementCamera, MovementMode, MovementModes, WalkMode, WalkStance};
use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSwitching};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_math::Vector3d;
use xrf_math::glam::Vec3;
use xrf_physics::{PhysicsLevel, PhysicsWorld};
use xrf_spawn::ClsId;

use crate::state::game_level_state::GameLevelState;
use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;
use crate::time::game_clock::GameClock;
use crate::time::game_time::GameTime;

/// The level the tests' objects stand on.
pub(crate) const LEVEL: u8 = 108;

/// How many stalkers stand in the field.
pub(crate) const STALKERS: u16 = 50;

/// The player's object, past the field of stalkers.
pub(crate) fn player_id() -> ObjectId {
  ObjectId::of_index(u32::from(STALKERS))
}

pub(crate) fn record(name: String, class: ClsId, position: Vec3, spawn_id: u16) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name,
    class,
    position,
    direction: Vec3::Z,
    game_vertex: Some(0),
    level_vertex: Some(0),
    story_id: None,
    spawn_id,
    switching: ObjectSwitching::ByDistance,
  }
}

/// A new game on a field of objects every 20 m along `+z` on one level, the player flying at the origin looking along
/// them, in a physics world holding nothing.
pub(crate) fn state() -> GameState {
  state_on(
    PhysicsWorld::new().expect("a world"),
    Movement::new(
      MovementCamera::default(),
      MovementMode::default(),
      MovementModes::default(),
    ),
  )
}

/// The same new game on foot: the player standing at the origin of a floor at `y = 0`, 400 m across, looking along
/// the field.
pub(crate) fn walking_state() -> GameState {
  let eye: f32 = WalkMode::default().get_eye_height(WalkStance::Standing);

  state_on(
    floor(),
    Movement::new(
      MovementCamera::looking_at(Vec3::new(0.0, eye, 0.0), Vec3::new(0.0, eye, 1.0)),
      MovementMode::Walk,
      MovementModes::default(),
    ),
  )
}

fn state_on(physics: PhysicsWorld, movement: Movement) -> GameState {
  let mut objects: ObjectRegistry = ObjectRegistry::default();

  for index in 0..STALKERS {
    objects.create(
      record(
        format!("stalker_{index}"),
        ClsId::AiStlS,
        Vec3::new(0.0, 0.0, f32::from(index) * 20.0),
        index,
      ),
      Some(LEVEL),
    );
  }

  let player: Option<ObjectId> = objects.create(
    record(String::from("actor"), ClsId::SActor, Vec3::ZERO, STALKERS),
    Some(LEVEL),
  );

  assert_eq!(player, Some(player_id()));

  GameState {
    progress: GameProgress {
      seed: 7,
      time: GameTime::new(GameClock::default()),
      objects,
      alife: Alife::new(AlifeSettings::default(), AlifeData::default()),
      player: player_id(),
    },
    level: GameLevelState {
      physics,
      graph_level: Some(LEVEL),
    },
    movement,
    hooks: AlifeHooks::of_defaults().expect("A-Life's hooks"),
  }
}

/// A world holding a floor at `y = 0`, 400 m across, its two faces wound as X-Ray winds them: clockwise from above.
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
      corner(-200.0, -200.0),
      corner(-200.0, 200.0),
      corner(200.0, 200.0),
      corner(200.0, -200.0),
    ],
    vec![face([0, 2, 1]), face([0, 3, 2])],
  )
  .expect("a floor");
  let materials: GameMtlFile = GameMtlFile::new(Vec::new(), Vec::new());

  PhysicsWorld::with_level(&PhysicsLevel::of_cform(&geometry, &materials).expect("the floor")).expect("a world")
}
