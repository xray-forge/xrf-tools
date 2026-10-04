use std::sync::Arc;

use glam::{Mat4, Vec3};

use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_effect_flags::ParticleEffectFlags;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_bounds::ParticleBounds;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_event::ParticleEvent;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_running_action::ParticleRunningAction;

/// `PS::CParticleEffect`: one playing copy of an effect definition, stepping its actions at the engine's fixed rate.
pub struct ParticleEffectInstance {
  definition: Arc<ParticleEffect>,
  flags: ParticleEffectFlags,
  actions: Vec<ParticleRunningAction>,
  pool: ParticlePool,
  /// `m_Frame.m_iFrameCount` and `m_fSpeed`, the engine's defaults where the effect has no frame chunk.
  frame_count: u32,
  frame_speed: f32,
  /// `m_fElapsedLimit`: seconds left before a time-limited effect stops itself.
  elapsed_limit: f32,
  /// `m_MemDT`: milliseconds accumulated towards the next step.
  memory_milliseconds: u32,
  is_playing: bool,
  is_stopping: bool,
  /// `m_InitialPosition`: where it was last placed.
  initial_position: Vec3,
  bounds: ParticleBounds,
}

impl ParticleEffectInstance {
  /// `SFrame::InitDefault`'s frame count and speed.
  const DEFAULT_FRAME_COUNT: u32 = 16;
  const DEFAULT_FRAME_SPEED: f32 = 24.0;

  /// The engine's step-count clamp: at most three steps (99 ms) per update, so a long pause does not run away.
  const MAX_STEPS: u32 = 3;

  /// `Compile`: a stopped copy of the definition, its random sequence started from a seed.
  pub fn new(definition: Arc<ParticleEffect>, seed: i32) -> Self {
    let flags: ParticleEffectFlags = ParticleEffectFlags(definition.flags);
    let (frame_count, frame_speed) = definition
      .frame
      .as_ref()
      .map_or((Self::DEFAULT_FRAME_COUNT, Self::DEFAULT_FRAME_SPEED), |frame| {
        (frame.frame_count, frame.frame_speed)
      });
    let mut pool: ParticlePool = ParticlePool::new(definition.max_particles as usize, seed);

    if flags.is(ParticleEffectFlags::RANDOM_FRAME) {
      pool = pool.with_random_frames(frame_count as i32);
    }

    if flags.is(ParticleEffectFlags::ANIMATED | ParticleEffectFlags::RANDOM_PLAYBACK) {
      pool = pool.with_random_playback();
    }

    Self {
      actions: definition.actions.iter().map(ParticleRunningAction::from).collect(),
      flags,
      pool,
      frame_count,
      frame_speed,
      elapsed_limit: Self::read_time_limit(&definition, flags).unwrap_or(0.0),
      memory_milliseconds: 0,
      is_playing: false,
      is_stopping: false,
      initial_position: Vec3::ZERO,
      bounds: ParticleBounds::around_point(Vec3::ZERO),
      definition,
    }
  }

  /// Reports births and deaths for an owning group to take, as `SetBirthDeadCB` does.
  pub(crate) fn with_events(mut self) -> Self {
    self.pool = self.pool.with_events();
    self
  }

  pub fn get_definition(&self) -> &ParticleEffect {
    &self.definition
  }

  pub fn get_flags(&self) -> ParticleEffectFlags {
    self.flags
  }

  pub fn get_pool(&self) -> &ParticlePool {
    &self.pool
  }

  pub fn get_bounds(&self) -> ParticleBounds {
    self.bounds
  }

  pub fn is_playing(&self) -> bool {
    self.is_playing
  }

  /// `GetTimeLimit`: the seconds it plays for, none when it loops.
  pub fn get_time_limit(&self) -> Option<f32> {
    Self::read_time_limit(&self.definition, self.flags)
  }

  /// `IsLooped`: a negative time limit, which is any effect without one.
  pub fn is_looped(&self) -> bool {
    self.get_time_limit().is_none_or(|limit| limit < 0.0)
  }

  /// `Play`.
  pub fn play(&mut self) {
    self.is_stopping = false;
    self.is_playing = true;

    for action in &mut self.actions {
      action.play();
    }
  }

  /// `Stop`: silences the sources; deferred, the live particles play out, otherwise they go at once.
  pub fn stop(&mut self, is_deferred: bool) {
    for action in &mut self.actions {
      action.stop();
    }

    if is_deferred {
      self.is_stopping = true;
    } else {
      self.pool.clear();
      self.is_playing = false;
    }
  }

  /// `UpdateParent` without `XFORM`: places every action in world space and hands sources the parent's velocity.
  pub fn update_parent(&mut self, matrix: &Mat4, velocity: Vec3) {
    self.initial_position = matrix.w_axis.truncate();

    for action in &mut self.actions {
      action.transform(matrix, velocity);
    }
  }

  /// The births and deaths since last taken, for an owner that asked for them.
  pub(crate) fn take_events(&mut self) -> Vec<ParticleEvent> {
    self.pool.take_events()
  }

  /// `OnFrame`: accumulates the milliseconds passed and takes whole steps of the engine's length, at most three.
  pub fn update(&mut self, frame_milliseconds: u32, rules: &ParticleEngineRules) {
    if !self.is_playing {
      self.bounds = ParticleBounds::around_point(self.initial_position);

      return;
    }

    let step_milliseconds: u32 = rules.get_step_milliseconds();
    let step_seconds: f32 = rules.get_step_seconds();
    let mut steps: u32 = 0;

    self.memory_milliseconds += frame_milliseconds;

    if self.memory_milliseconds >= step_milliseconds {
      steps = (self.memory_milliseconds / step_milliseconds).min(Self::MAX_STEPS);
      self.memory_milliseconds %= step_milliseconds;
    }

    for _ in 0..steps {
      if let Some(time_limit) = self.get_time_limit()
        && !self.is_stopping
      {
        self.elapsed_limit -= step_seconds;

        if self.elapsed_limit < 0.0 {
          self.elapsed_limit = time_limit;
          self.stop(true);

          break;
        }
      }

      let mut step: ParticleActionStep = ParticleActionStep::new(step_seconds, rules);

      for action in &mut self.actions {
        action.execute(&mut self.pool, &mut step);
      }

      if self
        .flags
        .is(ParticleEffectFlags::FRAMED | ParticleEffectFlags::ANIMATED)
      {
        self.animate(step_seconds);
      }

      // todo: Collide particles with the level's collision mesh when the effect sets `dfCollision`.

      if !self.pool.is_empty() {
        self.bounds = self.measure_bounds();
      }

      if self.is_stopping && self.pool.is_empty() {
        self.is_playing = false;
        self.is_stopping = false;

        break;
      }
    }
  }

  /// `ExecuteAnimate`: advances each particle's frame by the speed, backwards where it plays reversed, wrapping.
  fn animate(&mut self, dt: f32) {
    let count: f32 = self.frame_count as f32;
    let advance: f32 = self.frame_speed * dt;

    for m in self.pool.get_particles_mut() {
      let mut frame: f32 = m.frame as f32 / 255.0 + if m.is_reversed { -advance } else { advance };

      if frame > count {
        frame -= count;
      }

      if frame < 0.0 {
        frame += count;
      }

      m.frame = (frame * 255.0).floor() as u16;
    }
  }

  /// The box of the particles' positions grown by the largest size of any.
  fn measure_bounds(&self) -> ParticleBounds {
    let mut min: Vec3 = Vec3::splat(f32::MAX);
    let mut max: Vec3 = Vec3::splat(f32::MIN);
    let mut size: f32 = 0.0;

    for m in self.pool.get_particles() {
      min = min.min(m.position);
      max = max.max(m.position);
      size = size.max(m.size.max_element());
    }

    ParticleBounds {
      min: min - Vec3::splat(size),
      max: max + Vec3::splat(size),
    }
  }

  fn read_time_limit(definition: &ParticleEffect, flags: ParticleEffectFlags) -> Option<f32> {
    if flags.is(ParticleEffectFlags::TIME_LIMIT) {
      Some(definition.time_limit.unwrap_or(0.0))
    } else {
      None
    }
  }
}
