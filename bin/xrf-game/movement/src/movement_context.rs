use xrf_game_objects::ObjectRecord;
use xrf_physics::PhysicsWorld;

use crate::event::movement_event::MovementEvent;

/// What movement is handed of the rest of the game each step: the physics world the actor walks in, the player's own
/// record in the object registry, which it writes where the player stands, and where it tells its events.
pub struct MovementContext<'a> {
  pub physics: &'a mut PhysicsWorld,
  pub player: &'a mut ObjectRecord,
  pub events: &'a mut Vec<MovementEvent>,
}
