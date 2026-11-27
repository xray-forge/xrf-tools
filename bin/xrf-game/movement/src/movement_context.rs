use xrf_physics::PhysicsWorld;

/// What movement is handed of the rest of the game each step: the physics world the actor walks in.
pub struct MovementContext<'a> {
  pub physics: &'a mut PhysicsWorld,
}
