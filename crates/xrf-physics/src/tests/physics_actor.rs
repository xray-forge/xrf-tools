use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_math::Vector3d;
use xrf_math::glam::Vec3;

use crate::actor::physics_actor_id::PhysicsActorId;
use crate::actor::physics_actor_settings::PhysicsActorSettings;
use crate::actor::physics_actor_state::PhysicsActorState;
use crate::actor::physics_ground::PhysicsGround;
use crate::level::physics_level::PhysicsLevel;
use crate::world::physics_world::PhysicsWorld;

/// The game's fixed step: 100 Hz, as X-Ray steps its physics.
const STEP: f32 = 0.01;

/// The game material of a solid face: one the test library does not hold, which the level keeps.
const SOLID: u16 = 0;

/// A passable material, as bushes are.
const BUSH: u16 = 7;

/// A passable material the actor still collides with.
const ACTOR_OBSTACLE: u16 = 8;

/// A collision form built quad by quad, each face wound as X-Ray winds it: clockwise seen from its front, and made of
/// the material set last.
#[derive(Default)]
struct Form {
  vertices: Vec<Vector3d<f32>>,
  faces: Vec<LevelCformFace>,
  material: u16,
}

impl Form {
  /// Makes the quads added next of `material`.
  fn made_of(&mut self, material: u16) -> &mut Self {
    self.material = material;
    self
  }

  /// A wall 3 m high across the way at `x = 2`, facing the origin.
  fn add_wall(&mut self) -> &mut Self {
    self.add_quad(
      [
        Vec3::new(2.0, 0.0, -5.0),
        Vec3::new(2.0, 0.0, 5.0),
        Vec3::new(2.0, 3.0, 5.0),
        Vec3::new(2.0, 3.0, -5.0),
      ],
      Vec3::NEG_X,
    )
  }

  /// Adds the quad of `corners`, in order around it, facing `normal`.
  fn add_quad(&mut self, corners: [Vec3; 4], normal: Vec3) -> &mut Self {
    let base: u32 = u32::try_from(self.vertices.len()).expect("a small form");
    let is_counter_clockwise: bool = (corners[1] - corners[0]).cross(corners[2] - corners[0]).dot(normal) > 0.0;

    self.vertices.extend(corners.map(|corner| Vector3d {
      x: corner.x,
      y: corner.y,
      z: corner.z,
    }));

    for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
      // Clockwise seen from the front.
      let vertices: [u32; 3] = if is_counter_clockwise {
        [base + a, base + c, base + b]
      } else {
        [base + a, base + b, base + c]
      };

      self.faces.push(LevelCformFace {
        vertices,
        material: self.material,
        is_shadow_suppressed: false,
        is_wallmark_suppressed: false,
        sector: 0,
      });
    }

    self
  }

  /// A floor at `y = 0` 40 m across, centred on the origin.
  fn add_floor(&mut self) -> &mut Self {
    self.add_quad(
      [
        Vec3::new(-20.0, 0.0, -20.0),
        Vec3::new(20.0, 0.0, -20.0),
        Vec3::new(20.0, 0.0, 20.0),
        Vec3::new(-20.0, 0.0, 20.0),
      ],
      Vec3::Y,
    )
  }

  /// A platform `height` high from `x = 2` on, its top and its front wall facing the origin.
  fn add_platform(&mut self, height: f32) -> &mut Self {
    self
      .add_quad(
        [
          Vec3::new(2.0, height, -5.0),
          Vec3::new(12.0, height, -5.0),
          Vec3::new(12.0, height, 5.0),
          Vec3::new(2.0, height, 5.0),
        ],
        Vec3::Y,
      )
      .add_quad(
        [
          Vec3::new(2.0, 0.0, -5.0),
          Vec3::new(2.0, 0.0, 5.0),
          Vec3::new(2.0, height, 5.0),
          Vec3::new(2.0, height, -5.0),
        ],
        Vec3::NEG_X,
      )
  }

  fn build(&self) -> PhysicsWorld {
    let geometry: LevelCformGeometry =
      LevelCformGeometry::new(self.vertices.clone(), self.faces.clone()).expect("a valid form");
    let material = |id: u16, flags: u32| GameMtlMaterial {
      flags,
      ..GameMtlMaterial::new(u32::from(id), format!("material_{id}"))
    };
    let materials: GameMtlFile = GameMtlFile::new(
      vec![
        material(BUSH, GameMtlMaterial::FLAG_PASSABLE),
        material(
          ACTOR_OBSTACLE,
          GameMtlMaterial::FLAG_PASSABLE | GameMtlMaterial::FLAG_ACTOR_OBSTACLE,
        ),
      ],
      Vec::new(),
    );

    PhysicsWorld::with_level(&PhysicsLevel::of_cform(&geometry, &materials).expect("the level")).expect("a world")
  }
}

/// Moves `actor` at `velocity` for `ticks`, falling under gravity while unsupported, and answers where it ends.
fn walk(world: &mut PhysicsWorld, actor: PhysicsActorId, velocity: Vec3, ticks: usize) -> PhysicsActorState {
  let mut fall: f32 = 0.0;
  let mut state: PhysicsActorState = world.get_actor(actor).expect("the actor");

  for _ in 0..ticks {
    fall = match state.ground {
      PhysicsGround::Standing => 0.0,
      PhysicsGround::Steep | PhysicsGround::Air => fall + PhysicsWorld::GRAVITY.y * STEP,
    };
    state = world
      .move_actor(actor, velocity + Vec3::Y * fall, STEP)
      .expect("a move");
  }

  state
}

#[test]
fn actor_dropped_above_the_floor_lands_and_stands() {
  let mut world: PhysicsWorld = Form::default().add_floor().build();
  let actor: PhysicsActorId = world
    .create_actor(&PhysicsActorSettings::default(), Vec3::new(0.0, 2.0, 0.0))
    .expect("the actor");
  let state: PhysicsActorState = walk(&mut world, actor, Vec3::ZERO, 200);

  assert_eq!(state.ground, PhysicsGround::Standing);
  assert!(state.feet.y.abs() < 0.05, "the feet stand at {}", state.feet.y);
}

#[test]
fn actor_walks_the_floor_at_its_speed() {
  let mut world: PhysicsWorld = Form::default().add_floor().build();
  let actor: PhysicsActorId = world
    .create_actor(&PhysicsActorSettings::default(), Vec3::ZERO)
    .expect("the actor");
  let state: PhysicsActorState = walk(&mut world, actor, Vec3::new(0.0, 0.0, 3.0), 100);

  assert_eq!(state.ground, PhysicsGround::Standing);
  assert!(
    (state.feet.z - 3.0).abs() < 0.05,
    "the actor walked to {}",
    state.feet.z
  );
}

#[test]
fn actor_climbs_what_xray_climbs_and_stops_at_what_it_does_not() {
  for (height, is_climbed) in [(0.3, true), (0.5, true), (0.6, false), (0.9, false)] {
    let mut world: PhysicsWorld = Form::default().add_floor().add_platform(height).build();
    let actor: PhysicsActorId = world
      .create_actor(&PhysicsActorSettings::default(), Vec3::ZERO)
      .expect("the actor");
    let state: PhysicsActorState = walk(&mut world, actor, Vec3::new(3.6, 0.0, 0.0), 200);

    assert_eq!(
      (state.feet.y - height).abs() < 0.05,
      is_climbed,
      "a {height} m platform: the feet end at {:?}",
      state.feet
    );
  }
}

#[test]
fn actor_walks_through_bushes_but_not_walls_or_actor_obstacles() {
  for (material, is_passed) in [(BUSH, true), (SOLID, false), (ACTOR_OBSTACLE, false)] {
    let mut world: PhysicsWorld = Form::default()
      .made_of(SOLID)
      .add_floor()
      .made_of(material)
      .add_wall()
      .build();
    let actor: PhysicsActorId = world
      .create_actor(&PhysicsActorSettings::default(), Vec3::ZERO)
      .expect("the actor");
    let state: PhysicsActorState = walk(&mut world, actor, Vec3::new(3.6, 0.0, 0.0), 200);

    assert_eq!(
      state.feet.x > 3.0,
      is_passed,
      "material {material}: the feet end at {:?}",
      state.feet
    );
  }
}

#[test]
fn names_the_material_the_actor_stands_on() {
  const MUD: u16 = 5;
  const CONCRETE: u16 = 9;

  let mut world: PhysicsWorld = Form::default()
    .made_of(MUD)
    .add_floor()
    .made_of(CONCRETE)
    .add_platform(0.3)
    .build();
  let actor: PhysicsActorId = world
    .create_actor(&PhysicsActorSettings::default(), Vec3::ZERO)
    .expect("the actor");

  assert_eq!(walk(&mut world, actor, Vec3::ZERO, 10).ground_material, Some(MUD));
  assert_eq!(
    walk(&mut world, actor, Vec3::new(2.0, 0.0, 0.0), 200).ground_material,
    Some(CONCRETE)
  );
}

#[test]
fn crouches_under_a_ceiling_and_stands_only_where_there_is_room() {
  let mut world: PhysicsWorld = Form::default()
    .add_floor()
    .add_quad(
      [
        Vec3::new(2.0, 1.5, -5.0),
        Vec3::new(2.0, 1.5, 5.0),
        Vec3::new(12.0, 1.5, 5.0),
        Vec3::new(12.0, 1.5, -5.0),
      ],
      Vec3::NEG_Y,
    )
    .build();
  let actor: PhysicsActorId = world
    .create_actor(&PhysicsActorSettings::default(), Vec3::ZERO)
    .expect("the actor");

  // Standing, the 1.5 m ceiling stops it; crouched to `ph_box1`'s 1.34 m, it goes under.
  assert!(walk(&mut world, actor, Vec3::new(3.0, 0.0, 0.0), 100).feet.x < 2.0);
  assert!(world.reshape_actor(actor, 1.34).expect("a crouch"));
  assert!(walk(&mut world, actor, Vec3::new(3.0, 0.0, 0.0), 100).feet.x > 4.0);
  assert!(!world.reshape_actor(actor, 1.8).expect("no room to stand"));

  // Out the far side it stands up again, its feet where they were.
  let crouched: PhysicsActorState = walk(&mut world, actor, Vec3::new(3.0, 0.0, 0.0), 300);

  assert!(crouched.feet.x > 12.5, "{:?}", crouched.feet);
  assert!(world.reshape_actor(actor, 1.8).expect("room to stand"));
  assert!(
    world
      .get_actor(actor)
      .expect("the actor")
      .feet
      .abs_diff_eq(crouched.feet, 1e-3)
  );
}

#[test]
fn finds_the_ground_below_a_point() {
  let world: PhysicsWorld = Form::default().add_floor().add_platform(0.5).build();

  assert_eq!(
    world.find_ground(Vec3::new(0.0, 3.0, 0.0), 10.0).expect("a ray"),
    Some(0.0)
  );
  assert_eq!(
    world.find_ground(Vec3::new(5.0, 3.0, 0.0), 10.0).expect("a ray"),
    Some(0.5)
  );
  assert_eq!(world.find_ground(Vec3::new(50.0, 3.0, 0.0), 10.0).expect("a ray"), None);
}
