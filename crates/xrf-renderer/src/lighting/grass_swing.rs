/// One of the grass's swings (`CDetailManager::swing_desc`): each wave's amplitude, the seconds each wind takes to turn
/// once, and how fast the waves run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrassSwing {
  pub amp1: f32,
  pub amp2: f32,
  pub rot1: f32,
  pub rot2: f32,
  pub speed: f32,
}

impl GrassSwing {
  /// `system.ltx`'s `[details]` normal swing.
  pub const NORMAL: Self = Self {
    amp1: 0.1,
    amp2: 0.05,
    rot1: 30.0,
    rot2: 1.0,
    speed: 2.0,
  };

  /// Its fast swing.
  pub const FAST: Self = Self {
    amp1: 0.35,
    amp2: 0.2,
    rot1: 5.0,
    rot2: 0.5,
    speed: 0.5,
  };

  /// `swing_current.lerp`: between this and another by a share.
  pub fn mix(&self, other: &Self, by: f32) -> Self {
    let lerp = |from: f32, to: f32| from + (to - from) * by;

    Self {
      amp1: lerp(self.amp1, other.amp1),
      amp2: lerp(self.amp2, other.amp2),
      rot1: lerp(self.rot1, other.rot1),
      rot2: lerp(self.rot2, other.rot2),
      speed: lerp(self.speed, other.speed),
    }
  }
}
