use std::time::Duration;

use glam::{Mat4, Vec3};
use xrf_engine_target::XrayEngine;
use xrf_particles::ParticleLibrary;
use xrf_renderer::{
  AmbientGust, ParticleEmitterProxy, PlacedEffect, RenderAmbient, RenderAmbientEffect, RenderSceneUpdate, WeatherRandom,
};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::contract::world_ambient_effect_report::WorldAmbientEffectReport;
use crate::contract::world_ambient_report::WorldAmbientReport;
use crate::level::ambient_frame::AmbientFrame;
use crate::level::ambient_wind::AmbientWind;

/// What the schedule's numbers are drawn from on every level open, so a driven capture plays the same effects.
const SEED: u64 = 0x5EED_AB1E;

/// The seeds the played effects' own simulations take, one after another, clear of the placed systems' seeds.
const FIRST_EFFECT_SEED: i32 = 1 << 24;

/// The weather's ambient effects, as `CGamePersistent::WeathersUpdate` plays them near the camera: one at a time,
/// outdoors, an ambient's effect picked at random once the wait drawn at the last one's start has passed, standing where
/// the camera stood plus its offset, stopped once its life time is up or the camera goes indoors, and gone once its
/// particles have died; and the wind each effect brings. The scene plays each effect at an emitter of its own and tells
/// when its particles have died.
pub struct LevelAmbientEffects {
  random: WeatherRandom,
  /// `ambient_effect_next_time` and `ambient_effect_stop_time`, in milliseconds.
  next_time: u64,
  stop_time: u64,
  /// `ambient_particles`.
  playing: Option<PlayingEffect>,
  played: i32,
  /// Whether the next update is to play an effect at once, whatever plays or is waited for.
  is_forced: bool,
  wind: AmbientWind,
}

/// The effect playing: its emitter, the names it is reported by, and whether it was stopped.
struct PlayingEffect {
  emitter: ProxyHandle<ParticleEmitterProxy>,
  name: String,
  particles: String,
  is_stopping: bool,
}

/// What an update of the effects reads besides the frame's ambients: the effects and groups the level has and the
/// engine they play by, the emitters the scene's posts name, and the emitters whose particles died.
pub struct AmbientSystems<'a> {
  pub library: &'a ParticleLibrary,
  pub engine: XrayEngine,
  pub emitters: &'a mut ProxyAllocator<ParticleEmitterProxy>,
  pub finished: &'a [ProxyHandle<ParticleEmitterProxy>],
}

impl Default for LevelAmbientEffects {
  fn default() -> Self {
    Self {
      random: WeatherRandom::new(SEED),
      next_time: 0,
      stop_time: 0,
      playing: None,
      played: 0,
      is_forced: false,
      wind: AmbientWind::default(),
    }
  }
}

impl LevelAmbientEffects {
  /// Lets an effect whose particles died go, blows the wind on, starts an effect when one is due, and stops it once its
  /// time is up or indoors, in the order the engine takes them each frame, posting what the scene plays. `eye` is where
  /// the camera stands, in engine space; `now` is the effects' clock, in milliseconds.
  pub fn update(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    mut systems: AmbientSystems<'_>,
    frame: Option<AmbientFrame<'_>>,
    (eye, is_indoors): (Vec3, bool),
    now: u64,
  ) {
    let time: f32 = now as f32 / 1000.0;

    if let Some(playing) = &self.playing
      && systems.finished.contains(&playing.emitter)
    {
      self.release(updates, systems.emitters);
    }

    self.wind.blow(time);

    if self.is_forced {
      self.is_forced = false;
      self.release(updates, systems.emitters);
      self.next_time = 0;
    }

    if !is_indoors
      && self.playing.is_none()
      && now > self.next_time
      && let Some(frame) = frame
    {
      self.start(updates, &mut systems, frame, (eye, now));
    }

    self.wind.rise(time);

    if is_indoors || now >= self.stop_time {
      // `Stop()`, deferred: it emits no more, and what it emitted lives on.
      if let Some(playing) = &mut self.playing
        && !playing.is_stopping
      {
        playing.is_stopping = true;
        updates.push(RenderSceneUpdate::StopEffect {
          handle: playing.emitter,
          effect: PlacedEffect::Idle,
          is_deferred: true,
        });
      }

      self.wind.calm();
    }

    self.wind.fall(time);
  }

  /// The wind as it blows this frame.
  pub fn get_gust(&self) -> AmbientGust {
    self.wind.get_gust()
  }

  /// Ends what plays at once and plays an effect on the next update, outdoors, without waiting.
  pub fn play_now(&mut self) {
    self.is_forced = true;
  }

  /// Where they stand at a moment of the particles' clock: what plays and how much of its life is left, and how long
  /// until the next may start.
  pub fn report(&self, now: u64, is_indoors: bool) -> WorldAmbientReport {
    WorldAmbientReport {
      effect: self.playing.as_ref().map(|playing| WorldAmbientEffectReport {
        name: playing.name.clone(),
        particles: playing.particles.clone(),
        remaining: if playing.is_stopping {
          0.0
        } else {
          self.stop_time.saturating_sub(now) as f32 / 1000.0
        },
      }),
      is_indoors,
      wait: self.next_time.saturating_sub(now) as f32 / 1000.0,
      played: self.played as u32,
    }
  }

  /// Takes the effect playing out of the scene, its particles with it.
  fn release(&mut self, updates: &mut Vec<RenderSceneUpdate>, emitters: &mut ProxyAllocator<ParticleEmitterProxy>) {
    if let Some(playing) = self.playing.take() {
      emitters.free(playing.emitter);
      updates.push(RenderSceneUpdate::RemoveEmitter(playing.emitter));
    }
  }

  /// Plays an effect of the ambient the frame plays, `get_rnd_effect`, and draws when the next may start,
  /// `get_rnd_effect_time`; an ambient without effects plays none and draws nothing.
  fn start(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    systems: &mut AmbientSystems<'_>,
    frame: AmbientFrame<'_>,
    (eye, now): (Vec3, u64),
  ) {
    // `Current[data_set]`, drawn only when the keyframes' ambients differ and only here, where it is read, so that the
    // run of numbers does not depend on the frame rate.
    let name: Option<&str> = if frame.ambients.is_settled() {
      frame.ambients.names[0].as_deref()
    } else {
      frame.ambients.pick(self.random.next_fraction())
    };
    let Some(ambient) = name.and_then(|name| frame.level.ambients.get(name)) else {
      return;
    };

    if ambient.effects.is_empty() {
      return;
    }

    let chosen: &str = &ambient.effects[self.draw_index(ambient.effects.len())];

    self.next_time = now + self.draw_period(ambient);

    let Some(effect) = frame.level.ambient_effects.get(chosen) else {
      log::warn!("The ambient effect '{chosen}' is not described");

      return;
    };

    self.stop_time = now + effect.life_time.as_millis() as u64;
    self.wind.start(effect, now as f32 / 1000.0);

    let offset: Vec3 = self.draw_offset(effect, systems.engine);

    self.played += 1;

    // `CParticlesObject::Create`, then `play_at_pos`; an effect the library lacks plays nothing.
    let library: &ParticleLibrary = systems.library;

    if library.get_effect(&effect.particles).is_none() && library.get_group(&effect.particles).is_none() {
      log::warn!(
        "The ambient effect '{chosen}' plays '{}', which no effect or group is",
        effect.particles
      );

      return;
    }

    log::info!(
      "Ambient effect '{chosen}' plays '{}' for {:?}, the next in {} ms at the soonest",
      effect.particles,
      effect.life_time,
      self.next_time - now
    );

    let emitter: ProxyHandle<ParticleEmitterProxy> = systems.emitters.allocate();

    updates.push(RenderSceneUpdate::AddEmitter {
      handle: emitter,
      transform: Mat4::from_translation(eye + offset),
      seed: FIRST_EFFECT_SEED + self.played,
    });
    updates.push(RenderSceneUpdate::PlayEffect {
      handle: emitter,
      effect: PlacedEffect::Idle,
      name: effect.particles.clone(),
    });
    self.playing = Some(PlayingEffect {
      emitter,
      name: chosen.to_owned(),
      particles: effect.particles.clone(),
      is_stopping: false,
    });
  }

  /// `Random.randI(count)`.
  fn draw_index(&mut self, count: usize) -> usize {
    ((self.random.next_fraction() * count as f32) as usize).min(count - 1)
  }

  /// `Random.randI(min, max)` over the ambient's period, in milliseconds.
  fn draw_period(&mut self, ambient: &RenderAmbient) -> u64 {
    let (least, most): (Duration, Duration) = ambient.period;
    let (least, most): (u64, u64) = (least.as_millis() as u64, most.as_millis() as u64);

    least + ((most.saturating_sub(least)) as f32 * self.random.next_fraction()) as u64
  }

  /// The effect's offset, which Monolith moves half a metre to five aside on each level axis, either way.
  fn draw_offset(&mut self, effect: &RenderAmbientEffect, engine: XrayEngine) -> Vec3 {
    let mut offset: Vec3 = Vec3::from(effect.offset);

    if engine == XrayEngine::Extended {
      offset.x += self.draw_aside();
      offset.z += self.draw_aside();
    }

    offset
  }

  /// `Random.randF(0.5f, 5.f) * (Random.randF(0.f, 1.f) < 0.5f ? -1.f : 1.f)`.
  fn draw_aside(&mut self) -> f32 {
    let distance: f32 = self.random.between(0.5, 5.0);

    if self.random.next_fraction() < 0.5 {
      -distance
    } else {
      distance
    }
  }
}
