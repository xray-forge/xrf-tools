use std::collections::HashMap;
use std::sync::Arc;

use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_group::ParticleGroup;
use crate::particles_file::ParticlesFile;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_group_instance::ParticleGroupInstance;
use crate::simulation::particle_instance::ParticleInstance;

/// `CPSLibrary`: a `particles.xr`'s effects and groups by their exact, case-sensitive names.
#[derive(Default)]
pub struct ParticleLibrary {
  effects: HashMap<String, Arc<ParticleEffect>>,
  groups: HashMap<String, Arc<ParticleGroup>>,
}

impl ParticleLibrary {
  pub fn get_effect(&self, name: &str) -> Option<&Arc<ParticleEffect>> {
    self.effects.get(name)
  }

  pub fn get_group(&self, name: &str) -> Option<&Arc<ParticleGroup>> {
    self.groups.get(name)
  }

  pub fn get_effect_count(&self) -> usize {
    self.effects.len()
  }

  pub fn get_group_count(&self) -> usize {
    self.groups.len()
  }

  /// `model_CreatePE`: a stopped copy of a named effect.
  pub fn create_effect(&self, name: &str, seed: i32) -> Option<ParticleEffectInstance> {
    self
      .effects
      .get(name)
      .map(|effect| ParticleEffectInstance::new(effect.clone(), seed))
  }

  /// `model_CreateParticles`: a stopped copy of the effect by that name, or else of the group.
  pub fn create(&self, name: &str, seed: i32) -> Option<ParticleInstance> {
    if let Some(effect) = self.create_effect(name, seed) {
      return Some(ParticleInstance::Effect(effect));
    }

    self
      .groups
      .get(name)
      .map(|group| ParticleInstance::Group(ParticleGroupInstance::new(group.clone(), self, seed)))
  }
}

impl From<ParticlesFile> for ParticleLibrary {
  fn from(file: ParticlesFile) -> Self {
    Self {
      effects: file
        .effects
        .effects
        .into_iter()
        .map(|effect| (effect.name.clone(), Arc::new(effect)))
        .collect(),
      groups: file
        .groups
        .groups
        .into_iter()
        .map(|group| (group.name.clone(), Arc::new(group)))
        .collect(),
    }
  }
}
