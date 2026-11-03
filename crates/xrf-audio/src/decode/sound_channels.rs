/// The channels a sound carries. X-Ray sounds are mono, played in 3D, or stereo, which both engines always play in 2D;
/// a sound with more channels is refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SoundChannels {
  Mono,
  Stereo,
}

impl SoundChannels {
  /// The layout of `count` channels, or `None` for a count the engines do not play.
  pub fn from_count(count: usize) -> Option<Self> {
    match count {
      1 => Some(Self::Mono),
      2 => Some(Self::Stereo),
      _ => None,
    }
  }

  pub fn get_count(self) -> usize {
    match self {
      Self::Mono => 1,
      Self::Stereo => 2,
    }
  }
}
