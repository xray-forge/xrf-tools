use glam::Vec3;

use crate::host::sound::world_sound::WorldSound;

/// An ambient effect's sound starting this frame: the effect's section and where it plays, in engine space.
pub(crate) struct EffectSound<'a> {
  pub effect: &'a str,
  pub sound: &'a WorldSound,
  pub position: Vec3,
}
