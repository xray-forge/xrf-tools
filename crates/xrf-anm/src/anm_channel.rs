/// One channel an object motion animates, `EChannelType` (`xrCore/Animation/Motion.hpp`), declared in the engine's
/// order: the heading before the pitch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnmChannel {
  PositionX,
  PositionY,
  PositionZ,
  RotationHeading,
  RotationPitch,
  RotationBank,
}

impl AnmChannel {
  /// Every channel, in the engine's order, which is also the order `AnmFile::channels` holds them in.
  pub const ALL: [Self; 6] = [
    Self::PositionX,
    Self::PositionY,
    Self::PositionZ,
    Self::RotationHeading,
    Self::RotationPitch,
    Self::RotationBank,
  ];

  /// What the channel is called in the terms an author would recognise.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::PositionX => "position x",
      Self::PositionY => "position y",
      Self::PositionZ => "position z",
      Self::RotationHeading => "rotation heading",
      Self::RotationPitch => "rotation pitch",
      Self::RotationBank => "rotation bank",
    }
  }

  /// Where the channel sits in `AnmFile::channels`.
  pub const fn get_index(self) -> usize {
    self as usize
  }
}

#[cfg(test)]
mod tests {
  use crate::AnmChannel;

  #[test]
  fn each_channel_sits_where_the_engine_numbers_it() {
    for (index, channel) in AnmChannel::ALL.into_iter().enumerate() {
      assert_eq!(channel.get_index(), index);
    }

    assert_eq!(AnmChannel::RotationHeading.get_index(), 3);
    assert_eq!(AnmChannel::RotationPitch.get_index(), 4);
  }
}
