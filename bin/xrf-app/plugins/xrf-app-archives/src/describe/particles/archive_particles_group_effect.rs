use std::collections::HashSet;

use serde::Serialize;
use xrf_particles::ParticleGroupEffect;

use crate::describe::particles::archive_particles_effect_name::ArchiveParticlesEffectName;

/// One slot of a group: the effect it plays, and the effects that effect starts with it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesGroupEffect {
  pub effect: ArchiveParticlesEffectName,
  pub on_birth: Option<ArchiveParticlesEffectName>,
  pub on_play: Option<ArchiveParticlesEffectName>,
  pub on_dead: Option<ArchiveParticlesEffectName>,
  /// Seconds into the group the slot starts and stops, `time_0` and `time_1`.
  pub from: f32,
  pub to: f32,
  pub flags: u32,
}

impl ArchiveParticlesGroupEffect {
  pub(super) fn of(effect: &ParticleGroupEffect, effects: &HashSet<&str>) -> Self {
    Self {
      effect: ArchiveParticlesEffectName::of(&effect.name, effects),
      on_birth: ArchiveParticlesEffectName::of_optional(&effect.on_birth_child_name, effects),
      on_play: ArchiveParticlesEffectName::of_optional(&effect.on_play_child_name, effects),
      on_dead: ArchiveParticlesEffectName::of_optional(&effect.on_dead_child_name, effects),
      from: effect.time_0,
      to: effect.time_1,
      flags: effect.flags,
    }
  }
}
