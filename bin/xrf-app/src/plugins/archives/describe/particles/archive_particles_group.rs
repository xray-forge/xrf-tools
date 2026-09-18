use std::collections::HashSet;

use serde::Serialize;
use xrf_particles::{ParticleGroup, ParticleGroupEffect};

/// One sequence of the library: the effects it plays and what each one starts alongside itself.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesGroup {
  pub name: String,
  /// Seconds the group runs for; zero where it declares no limit.
  pub time_limit: f32,
  pub effects: Vec<ArchiveParticlesGroupEffect>,
}

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

/// An effect a group names, and whether this library is where it is defined.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesEffectName {
  pub name: String,
  pub is_defined: bool,
}

impl ArchiveParticlesGroup {
  /// Every group of a library, each naming its effects against the ones the library defines.
  pub fn of_all(groups: &[ParticleGroup], effects: &HashSet<&str>) -> Vec<Self> {
    groups
      .iter()
      .map(|group| Self {
        name: group.name.clone(),
        time_limit: group.time_limit,
        effects: group
          .effects
          .iter()
          .map(|effect| ArchiveParticlesGroupEffect::of(effect, effects))
          .collect(),
      })
      .collect()
  }
}

impl ArchiveParticlesGroupEffect {
  fn of(effect: &ParticleGroupEffect, effects: &HashSet<&str>) -> Self {
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

impl ArchiveParticlesEffectName {
  fn of(name: &str, effects: &HashSet<&str>) -> Self {
    Self {
      name: name.to_owned(),
      is_defined: effects.contains(name),
    }
  }

  /// The same, for a slot that is free to name nothing - which most of the three child slots do.
  fn of_optional(name: &str, effects: &HashSet<&str>) -> Option<Self> {
    (!name.is_empty()).then(|| Self::of(name, effects))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use xrf_particles::{ParticleGroup, ParticleGroupEffect};

  use super::ArchiveParticlesGroup;

  fn group_effect(name: &str, on_play: &str) -> ParticleGroupEffect {
    ParticleGroupEffect {
      name: String::from(name),
      on_play_child_name: String::from(on_play),
      on_birth_child_name: String::new(),
      on_dead_child_name: String::new(),
      time_0: 0.0,
      time_1: 1.0,
      flags: 6,
    }
  }

  #[test]
  fn a_group_says_which_of_the_effects_it_names_the_library_defines() {
    let defined: HashSet<&str> = HashSet::from(["effects\\smoke"]);
    let groups: Vec<ParticleGroup> = vec![ParticleGroup {
      version: 3,
      name: String::from("anomaly\\burn"),
      flags: 0,
      time_limit: 0.0,
      effects: vec![group_effect("effects\\smoke", ""), group_effect("effects\\gone", "")],
      description: None,
      effects_old: None,
    }];

    let described: Vec<ArchiveParticlesGroup> = ArchiveParticlesGroup::of_all(&groups, &defined);

    assert!(described[0].effects[0].effect.is_defined);
    assert!(!described[0].effects[1].effect.is_defined);
  }

  #[test]
  fn a_child_slot_naming_nothing_is_absent_rather_than_empty() {
    let defined: HashSet<&str> = HashSet::from(["effects\\spark"]);
    let groups: Vec<ParticleGroup> = vec![ParticleGroup {
      version: 3,
      name: String::from("anomaly\\burn"),
      flags: 0,
      time_limit: 0.0,
      effects: vec![group_effect("effects\\smoke", "effects\\spark")],
      description: None,
      effects_old: None,
    }];

    let described: Vec<ArchiveParticlesGroup> = ArchiveParticlesGroup::of_all(&groups, &defined);

    assert_eq!(described[0].effects[0].on_birth, None);
    assert_eq!(described[0].effects[0].on_dead, None);
    assert!(described[0].effects[0].on_play.as_ref().is_some_and(|it| it.is_defined));
  }
}
