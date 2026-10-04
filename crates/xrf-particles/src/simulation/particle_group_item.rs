use glam::{Mat4, Vec3};

use crate::data::particle_group_effect::ParticleGroupEffect;
use crate::data::particle_group_effect_flags::ParticleGroupEffectFlags;
use crate::simulation::particle::Particle;
use crate::simulation::particle_bounds::ParticleBounds;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_event::ParticleEvent;
use crate::simulation::particle_library::ParticleLibrary;
use crate::simulation::particle_random::ParticleRandom;

/// `CParticleGroup::SItem`: a group's effect, the children following its particles, and its free children.
pub(crate) struct ParticleGroupItem {
  effect: Option<ParticleEffectInstance>,
  /// `_children_related`: none where the named child is not in the library, so the indices still match.
  related: Vec<Option<ParticleEffectInstance>>,
  free: Vec<ParticleEffectInstance>,
  /// Draws each child's seed.
  seeds: ParticleRandom,
}

impl ParticleGroupItem {
  pub fn new(effect: Option<ParticleEffectInstance>, seed: i32) -> Self {
    Self {
      effect,
      related: Vec::new(),
      free: Vec::new(),
      seeds: ParticleRandom::new(seed),
    }
  }

  pub fn is_playing(&self) -> bool {
    self.effect.as_ref().is_some_and(ParticleEffectInstance::is_playing)
  }

  pub fn play(&mut self) {
    if let Some(effect) = &mut self.effect {
      effect.play();
    }
  }

  /// `Stop`: the effect and every child; without delay the children are dropped too.
  pub fn stop(&mut self, is_deferred: bool) {
    if let Some(effect) = &mut self.effect {
      effect.stop(is_deferred);
    }

    for child in self.related.iter_mut().flatten().chain(self.free.iter_mut()) {
      child.stop(is_deferred);
    }

    if !is_deferred {
      self.related.clear();
      self.free.clear();
    }
  }

  pub fn update_parent(&mut self, matrix: &Mat4, velocity: Vec3) {
    if let Some(effect) = &mut self.effect {
      effect.update_parent(matrix, velocity);
    }
  }

  /// Every effect it draws: its own, then the related and free children.
  pub fn get_effects(&self) -> impl Iterator<Item = &ParticleEffectInstance> {
    self
      .effect
      .iter()
      .chain(self.related.iter().flatten())
      .chain(self.free.iter())
  }

  /// `SItem::OnFrame`: steps the effect, starts and stops children for its births and deaths, then steps them.
  pub fn update(
    &mut self,
    frame_milliseconds: u32,
    definition: &ParticleGroupEffect,
    library: &ParticleLibrary,
    rules: &ParticleEngineRules,
    bounds: &mut Option<ParticleBounds>,
  ) -> bool {
    let flags: ParticleGroupEffectFlags = ParticleGroupEffectFlags(definition.flags);
    let mut is_playing: bool = false;
    let step_seconds: f32 = rules.get_step_seconds();

    if let Some(effect) = &mut self.effect {
      effect.update(frame_milliseconds, rules);

      let events: Vec<ParticleEvent> = effect.take_events();

      for event in events {
        self.replay(event, definition, library, step_seconds);
      }
    }

    if let Some(effect) = &self.effect
      && effect.is_playing()
    {
      is_playing = true;
      Self::merge(bounds, effect.get_bounds());

      if flags.is(ParticleGroupEffectFlags::ON_PLAY_CHILD) && !definition.on_play_child_name.is_empty() {
        for (particle, child) in effect.get_pool().get_particles().iter().zip(self.related.iter_mut()) {
          if let Some(child) = child {
            child.update_parent(
              &Mat4::from_translation(particle.position),
              Self::get_child_velocity(particle, step_seconds),
            );
          }
        }
      }
    }

    for child in self.related.iter_mut().flatten() {
      child.update(frame_milliseconds, rules);

      if child.is_playing() {
        is_playing = true;
        Self::merge(bounds, child.get_bounds());
      } else if flags.is(ParticleGroupEffectFlags::ON_PLAY_CHILD_REWIND) {
        child.play();
      }
    }

    self.free.retain_mut(|child| {
      child.update(frame_milliseconds, rules);

      if child.is_playing() {
        is_playing = true;
        Self::merge(bounds, child.get_bounds());
      }

      child.is_playing()
    });

    is_playing
  }

  /// `OnGroupParticleBirth` and `OnGroupParticleDead`, in the order the effect reported them.
  fn replay(
    &mut self,
    event: ParticleEvent,
    definition: &ParticleGroupEffect,
    library: &ParticleLibrary,
    step_seconds: f32,
  ) {
    let flags: ParticleGroupEffectFlags = ParticleGroupEffectFlags(definition.flags);

    match event {
      ParticleEvent::Birth(particle) => {
        if flags.is(ParticleGroupEffectFlags::ON_BIRTH_CHILD) {
          self.start_free_child(&definition.on_birth_child_name, &particle, library, step_seconds);
        }

        if flags.is(ParticleGroupEffectFlags::ON_PLAY_CHILD) {
          let child: Option<ParticleEffectInstance> =
            self.start_child(&definition.on_play_child_name, &particle, library, step_seconds);

          self.related.push(child);
        }
      }
      ParticleEvent::Death { index, particle } => {
        if flags.is(ParticleGroupEffectFlags::ON_PLAY_CHILD)
          && index < self.related.len()
          && let Some(mut child) = self.related.swap_remove(index)
        {
          child.stop(true);
          self.free.push(child);
        }

        if flags.is(ParticleGroupEffectFlags::ON_DEAD_CHILD) {
          self.start_free_child(&definition.on_dead_child_name, &particle, library, step_seconds);
        }
      }
    }
  }

  /// `StartFreeChild`: only an effect that ends by itself, as the engine refuses a looped one.
  fn start_free_child(&mut self, name: &str, particle: &Particle, library: &ParticleLibrary, step_seconds: f32) {
    if let Some(child) = self.start_child(name, particle, library, step_seconds) {
      if child.is_looped() {
        log::warn!("Particle effect '{name}' loops, which a group cannot start as a free child");
      } else {
        self.free.push(child);
      }
    }
  }

  /// A named child played at the particle, moving at its velocity over the last step.
  fn start_child(
    &mut self,
    name: &str,
    particle: &Particle,
    library: &ParticleLibrary,
    step_seconds: f32,
  ) -> Option<ParticleEffectInstance> {
    let seed: i32 = self.seeds.next_integer();
    let mut child: ParticleEffectInstance = library.create_effect(name, seed)?;

    child.play();
    child.update_parent(
      &Mat4::from_translation(particle.position),
      Self::get_child_velocity(particle, step_seconds),
    );

    Some(child)
  }

  /// `(pos - posB) / fDT_STEP`: on Monolith the child's own step, which is the same step for every effect here.
  fn get_child_velocity(particle: &Particle, step_seconds: f32) -> Vec3 {
    (particle.position - particle.previous_position) / step_seconds
  }

  fn merge(bounds: &mut Option<ParticleBounds>, other: ParticleBounds) {
    *bounds = Some(bounds.map_or(other, |it| it.merge(&other)));
  }
}
