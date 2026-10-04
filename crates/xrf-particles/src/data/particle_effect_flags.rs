/// `CPEDef` flags (`ParticleEffectDef.h`), read as the engine reads them: `is` asks for every bit of a mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleEffectFlags(pub u32);

impl ParticleEffectFlags {
  pub const SPRITE: u32 = 1 << 0;
  pub const FRAMED: u32 = 1 << 10;
  pub const ANIMATED: u32 = 1 << 11;
  pub const RANDOM_FRAME: u32 = 1 << 12;
  pub const RANDOM_PLAYBACK: u32 = 1 << 13;
  pub const TIME_LIMIT: u32 = 1 << 14;
  pub const ALIGN_TO_PATH: u32 = 1 << 15;
  pub const COLLISION: u32 = 1 << 16;
  pub const COLLISION_DELETE: u32 = 1 << 17;
  pub const VELOCITY_SCALE: u32 = 1 << 18;
  pub const COLLISION_DYNAMIC: u32 = 1 << 19;
  pub const WORLD_ALIGN: u32 = 1 << 20;
  pub const FACE_ALIGN: u32 = 1 << 21;
  pub const CULLING: u32 = 1 << 22;
  pub const CULL_CCW: u32 = 1 << 23;

  /// Every flag with its name, in bit order.
  pub const NAMES: [(u32, &'static str); 15] = [
    (Self::SPRITE, "Sprite"),
    (Self::FRAMED, "Framed"),
    (Self::ANIMATED, "Animated"),
    (Self::RANDOM_FRAME, "RandomFrame"),
    (Self::RANDOM_PLAYBACK, "RandomPlayback"),
    (Self::TIME_LIMIT, "TimeLimit"),
    (Self::ALIGN_TO_PATH, "AlignToPath"),
    (Self::COLLISION, "Collision"),
    (Self::COLLISION_DELETE, "CollisionDel"),
    (Self::VELOCITY_SCALE, "VelocityScale"),
    (Self::COLLISION_DYNAMIC, "CollisionDyn"),
    (Self::WORLD_ALIGN, "WorldAlign"),
    (Self::FACE_ALIGN, "FaceAlign"),
    (Self::CULLING, "Culling"),
    (Self::CULL_CCW, "CullCCW"),
  ];

  /// `Flags32::is`: every bit of the mask is set.
  pub fn is(&self, mask: u32) -> bool {
    self.0 & mask == mask
  }

  /// The names of the bits set, in bit order, as the engine's `dfXXX` constants name them.
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
  use super::ParticleEffectFlags;

  #[test]
  fn asks_for_every_bit_of_a_mask() {
    let flags: ParticleEffectFlags = ParticleEffectFlags(ParticleEffectFlags::FRAMED);

    assert!(flags.is(ParticleEffectFlags::FRAMED));
    assert!(!flags.is(ParticleEffectFlags::FRAMED | ParticleEffectFlags::ANIMATED));
  }

  #[test]
  fn names_the_bits_set_in_bit_order() {
    let flags: ParticleEffectFlags = ParticleEffectFlags(
      ParticleEffectFlags::CULL_CCW | ParticleEffectFlags::SPRITE | ParticleEffectFlags::ALIGN_TO_PATH,
    );

    assert_eq!(flags.list_names(), ["Sprite", "AlignToPath", "CullCCW"]);
  }
}
