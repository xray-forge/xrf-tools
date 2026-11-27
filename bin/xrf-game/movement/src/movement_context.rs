use xrf_game_objects::ObjectRecord;
use xrf_physics::PhysicsWorld;

/// What movement is handed of the rest of the game each step: the physics world the actor walks in, and the player's
/// own record in the object registry, which it writes where the player stands.
pub struct MovementContext<'a> {
  pub physics: &'a mut PhysicsWorld,
  pub player: &'a mut ObjectRecord,
}
