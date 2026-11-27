use xrf_physics::PhysicsWorld;

/// The level played as the simulation steps it, built again for each level: its physics world, and the game graph's
/// level it is (none for a level the graph holds no vertices on).
#[derive(Debug)]
pub struct GameLevelState {
  pub physics: PhysicsWorld,
  pub graph_level: Option<u8>,
}
