use glam::{Mat4, Vec3};

use crate::simulation::particle_bounds::ParticleBounds;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_group_instance::ParticleGroupInstance;
use crate::simulation::particle_update_context::ParticleUpdateContext;

/// `IParticleCustom`: a playing effect or group, whichever a name resolved to.
pub enum ParticleInstance {
  Effect(ParticleEffectInstance),
  Group(ParticleGroupInstance),
}

impl ParticleInstance {
  pub fn play(&mut self) {
    match self {
      Self::Effect(effect) => effect.play(),
      Self::Group(group) => group.play(),
    }
  }

  pub fn stop(&mut self, is_deferred: bool) {
    match self {
      Self::Effect(effect) => effect.stop(is_deferred),
      Self::Group(group) => group.stop(is_deferred),
    }
  }

  pub fn update_parent(&mut self, matrix: &Mat4, velocity: Vec3) {
    match self {
      Self::Effect(effect) => effect.update_parent(matrix, velocity),
      Self::Group(group) => group.update_parent(matrix, velocity),
    }
  }

  /// `OnFrame`.
  pub fn update(&mut self, frame_milliseconds: u32, context: &ParticleUpdateContext) {
    match self {
      Self::Effect(effect) => effect.update(frame_milliseconds, context),
      Self::Group(group) => group.update(frame_milliseconds, context),
    }
  }

  pub fn is_playing(&self) -> bool {
    match self {
      Self::Effect(effect) => effect.is_playing(),
      Self::Group(group) => group.is_playing(),
    }
  }

  /// `GetTimeLimit`: an effect without a limit answers -1, as the engine's does.
  pub fn get_time_limit(&self) -> f32 {
    match self {
      Self::Effect(effect) => effect.get_time_limit().unwrap_or(-1.0),
      Self::Group(group) => group.get_time_limit(),
    }
  }

  pub fn get_bounds(&self) -> ParticleBounds {
    match self {
      Self::Effect(effect) => effect.get_bounds(),
      Self::Group(group) => group.get_bounds(),
    }
  }

  /// Every effect it draws, which for a group is each item's effect and children.
  pub fn get_effects(&self) -> Box<dyn Iterator<Item = &ParticleEffectInstance> + '_> {
    match self {
      Self::Effect(effect) => Box::new(std::iter::once(effect)),
      Self::Group(group) => Box::new(group.get_effects()),
    }
  }
}
