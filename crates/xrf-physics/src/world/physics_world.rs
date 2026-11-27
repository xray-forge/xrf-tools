use std::collections::BTreeMap;
use std::fmt;

use oxijolt::{
  BodySettings, CharacterId, CharacterSettings, ExtendedUpdateSettings, GroundState, Quat, QueryFilter, RayCast,
  WorldSettings,
};
use xrf_error::{XrfError, XrfResult};
use xrf_math::glam::Vec3;

use crate::actor::physics_actor_id::PhysicsActorId;
use crate::actor::physics_actor_settings::PhysicsActorSettings;
use crate::actor::physics_actor_state::PhysicsActorState;
use crate::actor::physics_ground::PhysicsGround;
use crate::jolt::jolt_vectors::JoltVectors;
use crate::level::physics_level::PhysicsLevel;

/// The game's physics: Jolt's world holding the level's collision and the actors walking it, stepped by the game
/// loop's fixed step. Nothing outside this crate names the engine.
pub struct PhysicsWorld {
  world: oxijolt::PhysicsWorld,
  /// Each actor's extended update (its step-up), by its id's value.
  actors: BTreeMap<u32, ExtendedUpdateSettings>,
}

impl PhysicsWorld {
  /// X-Ray's gravity, m/s², along `-y`: twice Earth's (`Physics.cpp`, `default_world_gravity`), which makes its jumps
  /// low and its falls quick, on the actor and every body alike.
  pub const GRAVITY: Vec3 = Vec3::new(0.0, -2.0 * 9.81, 0.0);

  /// An empty world.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine cannot start.
  pub fn new() -> XrfResult<Self> {
    let settings: WorldSettings = WorldSettings::default().gravity(JoltVectors::create_vector(Self::GRAVITY));

    Ok(Self {
      world: oxijolt::PhysicsWorld::new(settings).map_err(Self::create_error)?,
      actors: BTreeMap::new(),
    })
  }

  /// A world holding `level`'s collision.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine cannot start or refuses the level.
  pub fn with_level(level: &PhysicsLevel) -> XrfResult<Self> {
    let mut world: Self = Self::new()?;

    world
      .world
      .create_body(&level.shape, &BodySettings::new_static())
      .map_err(Self::create_error)?;
    world.world.optimize_broad_phase();

    Ok(world)
  }

  /// Moves the world's bodies on by `step` seconds.
  ///
  /// # Errors
  ///
  /// Returns an error when the engine rejects the step.
  pub fn step(&mut self, step: f32) -> XrfResult {
    let report = self.world.step(step).map_err(Self::create_error)?;

    if !report.is_complete() {
      log::warn!("Physics step dropped contacts its buffers had no room for: {report:?}");
    }

    Ok(())
  }

  /// An actor shaped by `settings`, its feet at `feet`, its contacts found so it stands where it is put.
  ///
  /// # Errors
  ///
  /// Returns an error when the settings or the place are out of the engine's range.
  pub fn create_actor(&mut self, settings: &PhysicsActorSettings, feet: Vec3) -> XrfResult<PhysicsActorId> {
    let character: CharacterSettings<'_> = CharacterSettings::humanoid(settings.height, settings.radius)
      .map_err(Self::create_error)?
      .mass(settings.mass)
      .max_slope_angle(settings.max_slope_degrees.to_radians());
    let id: CharacterId = self
      .world
      .create_character(&character, JoltVectors::create_position(feet), Quat::IDENTITY)
      .map_err(Self::create_error)?;

    self
      .world
      .refresh_character_contacts(id, &QueryFilter::new())
      .map_err(Self::create_error)?;
    self.actors.insert(
      id.to_raw(),
      ExtendedUpdateSettings::default().walk_stairs_step_up(oxijolt::Vec3::new(0.0, settings.step_up, 0.0)),
    );

    Ok(PhysicsActorId(id))
  }

  /// Removes `actor`.
  ///
  /// # Errors
  ///
  /// Returns an error when the world holds no such actor.
  pub fn remove_actor(&mut self, actor: PhysicsActorId) -> XrfResult {
    self.actors.remove(&actor.0.to_raw());
    self.world.remove_character(actor.0).map_err(Self::create_error)
  }

  /// Where `actor` stands now.
  ///
  /// # Errors
  ///
  /// Returns an error when the world holds no such actor.
  pub fn get_actor(&self, actor: PhysicsActorId) -> XrfResult<PhysicsActorState> {
    let character = self.world.character(actor.0).map_err(Self::create_error)?;

    Ok(PhysicsActorState {
      feet: JoltVectors::read_position(character.position()),
      ground: match character.ground_state() {
        GroundState::OnGround => PhysicsGround::Standing,
        GroundState::OnSteepGround => PhysicsGround::Steep,
        GroundState::NotSupported | GroundState::InAir => PhysicsGround::Air,
      },
    })
  }

  /// Moves `actor` at `velocity` (m/s, gravity not added) for `step` seconds against the level: sliding along walls,
  /// walking up steps, sticking to the floor going down; answers where it then stands.
  ///
  /// # Errors
  ///
  /// Returns an error when the world holds no such actor or the move is out of the engine's range.
  pub fn move_actor(&mut self, actor: PhysicsActorId, velocity: Vec3, step: f32) -> XrfResult<PhysicsActorState> {
    let update: ExtendedUpdateSettings = *self
      .actors
      .get(&actor.0.to_raw())
      .ok_or_else(|| XrfError::new_not_found_error("the world holds no such actor"))?;

    self
      .world
      .character_mut(actor.0)
      .map_err(Self::create_error)?
      .set_linear_velocity(JoltVectors::create_vector(velocity))
      .map_err(Self::create_error)?;
    self
      .world
      .update_character(
        actor.0,
        step,
        JoltVectors::create_vector(Self::GRAVITY),
        &update,
        &QueryFilter::new(),
      )
      .map_err(Self::create_error)?;

    self.get_actor(actor)
  }

  /// The height of the first surface straight below `point`, within `depth` metres; none where there is none.
  ///
  /// # Errors
  ///
  /// Returns an error when the ray is out of the engine's range.
  pub fn find_ground(&self, point: Vec3, depth: f32) -> XrfResult<Option<f32>> {
    let ray: RayCast = RayCast::new(
      JoltVectors::create_position(point),
      oxijolt::Vec3::new(0.0, -depth, 0.0),
    );

    Ok(
      self
        .world
        .cast_ray(&ray, &QueryFilter::new())
        .map_err(Self::create_error)?
        .map(|hit| point.y - hit.distance),
    )
  }

  fn create_error(error: impl fmt::Display) -> XrfError {
    XrfError::new_unexpected_error(format!("physics: {error}"))
  }
}

impl fmt::Debug for PhysicsWorld {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("PhysicsWorld")
      .field("actors", &self.actors.len())
      .finish_non_exhaustive()
  }
}
