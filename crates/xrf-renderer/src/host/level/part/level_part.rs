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
}

impl LevelPart {
  /// Every part, in declaration order.
  pub const ALL: [LevelPart; 8] = [
    LevelPart::Sectors,
    LevelPart::Spawn,
    LevelPart::Lights,
    LevelPart::Grass,
    LevelPart::Helpers,
    LevelPart::Particles,
    LevelPart::Weather,
    LevelPart::Motions,
  ];

  /// The parts a level's load waits for beside its textures, in the order its report names them.
  pub const LOADED: [LevelPart; 5] = [
    LevelPart::Sectors,
    LevelPart::Spawn,
    LevelPart::Lights,
    LevelPart::Particles,
    LevelPart::Grass,
  ];

  /// Its name, as logs and reports say it.
  pub fn get_name(self) -> &'static str {
    match self {
      LevelPart::Sectors => "sectors",
      LevelPart::Spawn => "spawn",
      LevelPart::Lights => "lights",
      LevelPart::Grass => "grass",
      LevelPart::Helpers => "helpers",
      LevelPart::Particles => "particles",
      LevelPart::Weather => "weather",
      LevelPart::Motions => "motions",
    }
  }

  /// Its bit in a set of parts.
  pub(super) fn get_bit(self) -> u16 {
    1 << self as u16
  }
}
