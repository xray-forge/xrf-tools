/// A part of a level the world reads and streams into its scene: what its reloads read again, what its progress counts
/// finished, and what its load report times.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelPart {
  /// Its sectors, streamed a few a frame.
  Sectors,
  /// Its spawned objects and their models, with the hemisphere cubes they are lit by.
  Spawn,
  Lights,
  /// Its grass, planted again by every view once read.
  Grass,
  /// The level objects the viewer draws helpers for.
  Helpers,
  /// Its particle systems, every placement played afresh.
  Particles,
  /// What every weather of the level plays with, the clock and the request of each viewport showing it kept.
  Weather,
  /// The object motions its moving zones follow, each read again the next time something it carries asks.
  Motions,
  /// Its AI map, which the debug views show over the ground.
  AiMap,
}

impl LevelPart {
  /// Every part, in declaration order.
  pub const ALL: [Self; 9] = [
    Self::Sectors,
    Self::Spawn,
    Self::Lights,
    Self::Grass,
    Self::Helpers,
    Self::Particles,
    Self::Weather,
    Self::Motions,
    Self::AiMap,
  ];

  /// The parts a level's load waits for beside its textures, in the order its report names them.
  pub const LOADED: [Self; 5] = [Self::Sectors, Self::Spawn, Self::Lights, Self::Particles, Self::Grass];

  /// Its name, as logs and reports say it.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Sectors => "sectors",
      Self::Spawn => "spawn",
      Self::Lights => "lights",
      Self::Grass => "grass",
      Self::Helpers => "helpers",
      Self::Particles => "particles",
      Self::Weather => "weather",
      Self::Motions => "motions",
      Self::AiMap => "ai map",
    }
  }

  /// Its bit in a set of parts.
  pub(super) fn get_bit(self) -> u16 {
    1 << self as u16
  }
}
