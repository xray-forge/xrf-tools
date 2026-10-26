/// `CPGDef::SEffect` flags (`ParticleGroup.h`), read as the engine reads them: `is` asks for every bit of a mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleGroupEffectFlags(pub u32);

impl ParticleGroupEffectFlags {
  pub const DEFERRED_STOP: u32 = 1 << 0;
  pub const ON_PLAY_CHILD: u32 = 1 << 1;
  pub const ENABLED: u32 = 1 << 2;
  pub const ON_PLAY_CHILD_REWIND: u32 = 1 << 4;
  pub const ON_BIRTH_CHILD: u32 = 1 << 5;
  pub const ON_DEAD_CHILD: u32 = 1 << 6;

  /// Every flag with its name, in bit order.
  pub const NAMES: [(u32, &'static str); 6] = [
    (Self::DEFERRED_STOP, "DefferedStop"),
    (Self::ON_PLAY_CHILD, "OnPlayChild"),
    (Self::ENABLED, "Enabled"),
    (Self::ON_PLAY_CHILD_REWIND, "OnPlayChildRewind"),
    (Self::ON_BIRTH_CHILD, "OnBirthChild"),
    (Self::ON_DEAD_CHILD, "OnDeadChild"),
  ];

  /// `Flags32::is`: every bit of the mask is set.
  pub fn is(&self, mask: u32) -> bool {
    self.0 & mask == mask
  }

  /// The names of the bits set, in bit order, as the engine's `flXXX` constants name them.
  pub fn list_names(&self) -> Vec<&'static str> {
    Self::NAMES
      .iter()
      .filter(|(bit, _)| self.is(*bit))
      .map(|(_, name)| *name)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::ParticleGroupEffectFlags;

  #[test]
  fn asks_for_every_bit_of_a_mask() {
    let flags: ParticleGroupEffectFlags =
      ParticleGroupEffectFlags(ParticleGroupEffectFlags::ENABLED | ParticleGroupEffectFlags::ON_PLAY_CHILD);

    assert!(flags.is(ParticleGroupEffectFlags::ENABLED));
    assert!(!flags.is(ParticleGroupEffectFlags::ENABLED | ParticleGroupEffectFlags::ON_DEAD_CHILD));
  }

  #[test]
  fn names_the_bits_set_in_bit_order() {
    let flags: ParticleGroupEffectFlags =
      ParticleGroupEffectFlags(ParticleGroupEffectFlags::ON_DEAD_CHILD | ParticleGroupEffectFlags::ENABLED);

    assert_eq!(flags.list_names(), ["Enabled", "OnDeadChild"]);
  }
}
