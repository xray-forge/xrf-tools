/// One bit of a motion definition's flag word, `ESMFlags` (`xrCore/Animation/Motion.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ArchiveOmfMotionFlag {
  /// Plays as an effect rather than a cycle, which is also what makes the definition's target a bone.
  Fx = 0,
  StopAtEnd = 1,
  NoMix = 2,
  SyncPart = 3,
  UseFootSteps = 4,
  RootMover = 5,
  Idle = 6,
  UseWeaponBone = 7,
}

impl ArchiveOmfMotionFlag {
  /// Every bit the engine names, in bit order.
  pub const NAMED: [Self; 8] = [
    Self::Fx,
    Self::StopAtEnd,
    Self::NoMix,
    Self::SyncPart,
    Self::UseFootSteps,
    Self::RootMover,
    Self::Idle,
    Self::UseWeaponBone,
  ];

  /// Engine identifier, which is the name the SDK and the sources spell the bit with.
  pub const fn label(self) -> &'static str {
    match self {
      Self::Fx => "esmFX",
      Self::StopAtEnd => "esmStopAtEnd",
      Self::NoMix => "esmNoMix",
      Self::SyncPart => "esmSyncPart",
      Self::UseFootSteps => "esmUseFootSteps",
      Self::RootMover => "esmRootMover",
      Self::Idle => "esmIdle",
      Self::UseWeaponBone => "esmUseWeaponBone",
    }
  }

  /// The word this bit sits in.
  pub const fn mask(self) -> u32 {
    1 << (self as u32)
  }

  /// Whether a definition's word carries this bit.
  pub const fn is_set_in(self, flags: u32) -> bool {
    flags & self.mask() != 0
  }

  /// The named bits a word carries, in bit order.
  pub fn set_in(flags: u32) -> Vec<String> {
    Self::NAMED
      .into_iter()
      .filter(|flag| flag.is_set_in(flags))
      .map(|flag| flag.label().to_owned())
      .collect()
  }

  /// Bits of a word that no name here claims, which is what keeps a bank written by another tool from reading as
  /// though it set nothing.
  pub const fn unnamed_in(flags: u32) -> u32 {
    let mut named: u32 = 0;
    let mut index: usize = 0;

    while index < Self::NAMED.len() {
      named |= Self::NAMED[index].mask();
      index += 1;
    }

    flags & !named
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveOmfMotionFlag;

  #[test]
  fn each_bit_sits_where_the_engine_declares_it() {
    assert_eq!(ArchiveOmfMotionFlag::Fx.mask(), 1);
    assert_eq!(ArchiveOmfMotionFlag::StopAtEnd.mask(), 1 << 1);
    assert_eq!(ArchiveOmfMotionFlag::UseWeaponBone.mask(), 1 << 7);
  }

  #[test]
  fn a_word_reports_the_bits_it_carries_in_bit_order() {
    // The commonest word in vanilla after `esmStopAtEnd` alone: an idle that blends across the partition.
    assert_eq!(
      ArchiveOmfMotionFlag::set_in(0b101_0010),
      vec![
        String::from("esmStopAtEnd"),
        String::from("esmUseFootSteps"),
        String::from("esmIdle")
      ]
    );
  }

  #[test]
  fn a_word_carrying_only_named_bits_leaves_no_residue() {
    assert_eq!(ArchiveOmfMotionFlag::unnamed_in(0xFF), 0);
    assert_eq!(ArchiveOmfMotionFlag::unnamed_in(0x100), 0x100);
  }
}
