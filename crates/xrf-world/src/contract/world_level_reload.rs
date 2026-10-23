/// What of a level its next source reads again, each from that source, while everything else stays as it was drawn.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldLevelReload {
  /// The spawned objects and their models, with the hemisphere cubes they are lit by.
  pub is_spawn: bool,
  pub is_lights: bool,
  /// The particle systems, every placement played afresh.
  pub is_particles: bool,
  /// What every weather of the level plays with, in each viewport showing it, the clock and the request kept.
  pub is_weather: bool,
  /// The object motions its moving zones follow, each read again the next time something it carries asks.
  pub is_motions: bool,
}

impl WorldLevelReload {
  /// Whether it reads nothing again.
  pub fn is_empty(&self) -> bool {
    *self == Self::default()
  }
}
