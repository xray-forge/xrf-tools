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

  /// `Flags32::is`: every bit of the mask is set.
  pub fn is(&self, mask: u32) -> bool {
    self.0 & mask == mask
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
}
