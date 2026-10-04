use std::sync::Arc;

use glam::{Mat4, Vec3};

use crate::data::particle_group::ParticleGroup;
use crate::data::particle_group_effect_flags::ParticleGroupEffectFlags;
use crate::simulation::particle_bounds::ParticleBounds;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_group_item::ParticleGroupItem;
use crate::simulation::particle_library::ParticleLibrary;
use crate::simulation::particle_random::ParticleRandom;

/// `PS::CParticleGroup`: a group's effects, each played from its `time0` to its `time1` on the group's clock.
pub struct ParticleGroupInstance {
  definition: Arc<ParticleGroup>,
  items: Vec<ParticleGroupItem>,
  /// `m_fTimeLimit` as loaded: the file's limit, or the latest `time1` where the file gives none.
  time_limit: f32,
  /// `m_CurrentTime`: seconds since the group played.
  current_time: f32,
  is_playing: bool,
  is_stopping: bool,
  initial_position: Vec3,
  bounds: ParticleBounds,
}

impl ParticleGroupInstance {
  /// `Compile`: a stopped group with one stopped copy of each of its effects the library holds.
  pub fn new(definition: Arc<ParticleGroup>, library: &ParticleLibrary, seed: i32) -> Self {
    let mut seeds: ParticleRandom = ParticleRandom::new(seed);
    let items: Vec<ParticleGroupItem> = definition
      .effects
      .iter()
      .map(|effect| {
        let instance: Option<ParticleEffectInstance> = library
          .create_effect(&effect.name, seeds.next_integer())
          .map(ParticleEffectInstance::with_events);

        ParticleGroupItem::new(instance, seeds.next_integer())
      })
      .collect();
    let time_limit: f32 = if definition.time_limit > 0.0 {
      definition.time_limit
    } else {
      definition
        .effects
        .iter()
        .fold(definition.time_limit, |limit, effect| limit.max(effect.time_1))
    };

    Self {
      definition,
      items,
      time_limit,
      current_time: 0.0,
      is_playing: false,
      is_stopping: false,
      initial_position: Vec3::ZERO,
      bounds: ParticleBounds::around_point(Vec3::ZERO),
    }
  }

  pub fn get_definition(&self) -> &ParticleGroup {
    &self.definition
  }

  pub fn get_bounds(&self) -> ParticleBounds {
    self.bounds
  }

  pub fn is_playing(&self) -> bool {
    self.is_playing
  }

  /// `GetTimeLimit`: zero or less when the group loops.
  pub fn get_time_limit(&self) -> f32 {
    self.time_limit
  }

  /// Every effect it draws, item by item.
  pub fn get_effects(&self) -> impl Iterator<Item = &ParticleEffectInstance> {
    self.items.iter().flat_map(ParticleGroupItem::get_effects)
  }

  /// `Play`: starts the clock; each effect starts as the clock passes its `time0`.
  pub fn play(&mut self) {
    self.current_time = 0.0;
    self.is_stopping = false;
    self.is_playing = true;
  }

  pub fn stop(&mut self, is_deferred: bool) {
    if is_deferred {
      self.is_stopping = true;
    } else {
      self.is_playing = false;
    }

    for item in &mut self.items {
      item.stop(is_deferred);
    }
  }

  pub fn update_parent(&mut self, matrix: &Mat4, velocity: Vec3) {
    self.initial_position = matrix.w_axis.truncate();

    for item in &mut self.items {
      item.update_parent(matrix, velocity);
    }
  }

  /// `OnFrame`: plays and stops effects whose times this frame crosses, then steps every item.
  pub fn update(&mut self, frame_milliseconds: u32, library: &ParticleLibrary, rules: &ParticleEngineRules) {
    if !self.is_playing {
      self.bounds = ParticleBounds::around_point(self.initial_position);

      return;
    }

    let now: f32 = self.current_time;
    let next: f32 = now + frame_milliseconds as f32 / 1000.0;

    for (effect, item) in self.definition.effects.iter().zip(self.items.iter_mut()) {
      let flags: ParticleGroupEffectFlags = ParticleGroupEffectFlags(effect.flags);

      if !flags.is(ParticleGroupEffectFlags::ENABLED) {
        continue;
      }

      if item.is_playing() {
        if now <= effect.time_1 && next >= effect.time_1 {
          item.stop(flags.is(ParticleGroupEffectFlags::DEFERRED_STOP));
        }
      } else if !self.is_stopping && now <= effect.time_0 && next >= effect.time_0 {
        item.play();
      }
    }

    self.current_time = next;

    if self.time_limit > 0.0 && self.current_time > self.time_limit && !self.is_stopping {
      self.stop(true);
    }

    let mut bounds: Option<ParticleBounds> = None;
    let mut is_any_playing: bool = false;

    for (effect, item) in self.definition.effects.iter().zip(self.items.iter_mut()) {
      is_any_playing |= item.update(frame_milliseconds, effect, library, rules, &mut bounds);
    }

    if self.is_stopping && !is_any_playing {
      self.is_playing = false;
      self.is_stopping = false;
    }

    if let Some(bounds) = bounds {
      self.bounds = bounds;
    }
  }
}
