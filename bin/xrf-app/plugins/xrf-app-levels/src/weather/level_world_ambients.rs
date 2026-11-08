use std::collections::HashMap;
use std::time::Duration;

use xrf_engine_target::XrayEngine;
use xrf_environment::{
  AmbientEffect, AmbientEffectKey, AmbientKey, EnvironmentCatalog, EnvironmentValue, SoundChannel, SoundChannelKey,
};
use xrf_world::{WorldAmbient, WorldAmbientEffect, WorldSound, WorldSoundChannel, WorldWindBlast};

/// A level's ambients and the effects they play near the camera, as the world takes them.
pub(crate) struct LevelWorldAmbients;

impl LevelWorldAmbients {
  /// A wind blast's ramp longer than this is read as this: past it a config holds no time a game could play out, and an
  /// infinite one would not fit a `Duration` at all.
  const LONGEST_BLAST_RAMP: f32 = 3600.0;

  /// Every ambient a keyframe may name, as it plays on a level: its own file's section over a shared one of the name.
  pub(crate) fn to_world(catalog: &EnvironmentCatalog, level: &str) -> HashMap<String, WorldAmbient> {
    catalog
      .ambients
      .iter()
      .filter_map(|shared| catalog.find_level_ambient(level, &shared.name))
      .map(|ambient| {
        let [least, most] = AmbientKey::get_effect_period(ambient, catalog.engine);

        (
          ambient.name.clone(),
          WorldAmbient {
            sound_channels: AmbientKey::list_channels(ambient, catalog.engine)
              .into_iter()
              .map(str::to_owned)
              .collect(),
            effects: ambient.get_list(AmbientKey::Effects).to_vec(),
            least_period: Self::to_milliseconds(least),
            most_period: Self::to_milliseconds(most),
          },
        )
      })
      .collect()
  }

  /// Every effect an ambient may play, `CEnvAmbient::create_effect`, its sound read by `read_sound`.
  pub(crate) fn to_world_effects(
    catalog: &EnvironmentCatalog,
    read_sound: &mut dyn FnMut(&str) -> WorldSound,
  ) -> HashMap<String, WorldAmbientEffect> {
    catalog
      .ambient_effects
      .iter()
      .map(|effect| {
        let sound: &str = effect.get_text(AmbientEffectKey::Sound, catalog.engine);
        let world: WorldAmbientEffect = WorldAmbientEffect {
          sound: (!sound.trim().is_empty()).then(|| read_sound(sound)),
          ..Self::to_world_effect(effect, catalog)
        };

        (effect.name.clone(), world)
      })
      .collect()
  }

  /// Every sound channel an ambient may play, `CEnvAmbient::SSndChannel::load`, its sounds read by `read_sound`.
  pub(crate) fn to_world_channels(
    catalog: &EnvironmentCatalog,
    read_sound: &mut dyn FnMut(&str) -> WorldSound,
  ) -> HashMap<String, WorldSoundChannel> {
    catalog
      .sound_channels
      .iter()
      .map(|channel| {
        let world: WorldSoundChannel = WorldSoundChannel {
          sounds: channel
            .get_list(SoundChannelKey::Sounds)
            .iter()
            .map(|sound| read_sound(sound))
            .collect(),
          distance: Self::to_channel_distance(channel, catalog.engine),
          period: Self::to_channel_period(channel, catalog.engine),
        };

        (channel.name.clone(), world)
      })
      .collect()
  }

  /// `m_sound_dist`: OpenXRay's `sound_dist`, nearest first, which either key still overrides, else the two keys.
  fn to_channel_distance(channel: &SoundChannel, engine: XrayEngine) -> [f32; 2] {
    if engine == XrayEngine::Vanilla && channel.has(SoundChannelKey::SoundDist) {
      let [a, b] = channel.get_vector::<2>(SoundChannelKey::SoundDist, engine);
      let mut distance: [f32; 2] = [a.min(b), a.max(b)];

      if channel.has(SoundChannelKey::MinDistance) {
        distance[0] = channel.get_number(SoundChannelKey::MinDistance, engine);
      }

      if channel.has(SoundChannelKey::MaxDistance) {
        distance[1] = channel.get_number(SoundChannelKey::MaxDistance, engine);
      }

      return distance;
    }

    [
      channel.get_number(SoundChannelKey::MinDistance, engine),
      channel.get_number(SoundChannelKey::MaxDistance, engine),
    ]
  }

  /// `m_sound_period` in milliseconds: OpenXRay's `sound_period` in seconds, four values (pre-Clear Sky) or two for
  /// both halves (Shadow of Chernobyl), each `period0`..`period3` written beside it overriding its value in seconds;
  /// else the four keys in milliseconds, as Call of Pripyat and Monolith write them.
  fn to_channel_period(channel: &SoundChannel, engine: XrayEngine) -> [u32; 4] {
    const KEYS: [SoundChannelKey; 4] = [
      SoundChannelKey::Period0,
      SoundChannelKey::Period1,
      SoundChannelKey::Period2,
      SoundChannelKey::Period3,
    ];
    let to_millis = |value: f64| value.max(0.0) as u32;
    let written: Option<&[f32]> = (engine == XrayEngine::Vanilla)
      .then(|| {
        channel
          .get(SoundChannelKey::SoundPeriod)
          .and_then(EnvironmentValue::as_vector)
      })
      .flatten();

    match written {
      Some(written) => {
        let seconds: [f32; 4] = match written {
          [least, most] => [*least, *most, *least, *most],
          _ => std::array::from_fn(|index| written.get(index).copied().unwrap_or(0.0)),
        };

        std::array::from_fn(|index| {
          let key: SoundChannelKey = KEYS[index];
          let value: f64 = if channel.has(key) {
            f64::from(channel.get_integer(key, engine))
          } else {
            f64::from(seconds[index])
          };

          to_millis(value * 1000.0)
        })
      }
      None => KEYS.map(|key| to_millis(f64::from(channel.get_integer(key, engine)))),
    }
  }

  /// The particles every effect of the catalog plays, which the level's particle surfaces must describe as well.
  pub(crate) fn list_particles(catalog: &EnvironmentCatalog) -> Vec<String> {
    catalog
      .ambient_effects
      .iter()
      .map(|effect| effect.get_text(AmbientEffectKey::Particles, catalog.engine).to_owned())
      .filter(|particles| !particles.is_empty())
      .collect()
  }

  /// One effect: its life in whole milliseconds as the engine floors it, and a blast only where `wind_blast_strength`
  /// is written, which otherwise stands at none from straight ahead.
  fn to_world_effect(effect: &AmbientEffect, catalog: &EnvironmentCatalog) -> WorldAmbientEffect {
    let engine: XrayEngine = catalog.engine;
    let wind_blast: WorldWindBlast = if effect.has(AmbientEffectKey::WindBlastStrength) {
      WorldWindBlast {
        strength: effect.get_number(AmbientEffectKey::WindBlastStrength, engine),
        longitude: effect
          .get_number(AmbientEffectKey::WindBlastLongitude, engine)
          .to_radians(),
        in_time: Self::to_seconds(effect.get_number(AmbientEffectKey::WindBlastInTime, engine)),
        out_time: Self::to_seconds(effect.get_number(AmbientEffectKey::WindBlastOutTime, engine)),
      }
    } else {
      WorldWindBlast::default()
    };

    WorldAmbientEffect {
      particles: effect.get_text(AmbientEffectKey::Particles, engine).to_owned(),
      life_time: Self::to_milliseconds(effect.get_number(AmbientEffectKey::LifeTime, engine)),
      offset: effect.get_vector::<3>(AmbientEffectKey::Offset, engine),
      wind_gust_factor: effect.get_number(AmbientEffectKey::WindGustFactor, engine),
      wind_blast,
      sound: None,
    }
  }

  /// Seconds as the engine keeps them in whole milliseconds, `iFloor(seconds * 1000)`.
  fn to_milliseconds(seconds: f32) -> Duration {
    Duration::from_millis((seconds * 1000.0).floor().max(0.0) as u64)
  }

  /// Seconds of a wind blast's ramp, none for a negative or unreadable value.
  fn to_seconds(seconds: f32) -> Duration {
    if seconds.is_nan() {
      Duration::ZERO
    } else {
      Duration::from_secs_f32(seconds.clamp(0.0, Self::LONGEST_BLAST_RAMP))
    }
  }
}
