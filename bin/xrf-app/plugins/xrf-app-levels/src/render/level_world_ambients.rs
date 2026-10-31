use std::collections::HashMap;
use std::time::Duration;

use xrf_engine_target::XrayEngine;
use xrf_environment::{AmbientEffect, AmbientEffectKey, AmbientKey, EnvironmentCatalog};
use xrf_world::{WorldAmbient, WorldAmbientEffect, WorldWindBlast};

/// A level's ambients and the effects they play near the camera, as the world takes them.
pub struct LevelWorldAmbients;

impl LevelWorldAmbients {
  /// A wind blast's ramp longer than this is read as this: past it a config holds no time a game could play out, and an
  /// infinite one would not fit a `Duration` at all.
  const LONGEST_BLAST_RAMP: f32 = 3600.0;

  /// Every ambient a keyframe may name, as it plays on a level: its own file's section over a shared one of the name.
  pub fn to_world(catalog: &EnvironmentCatalog, level: &str) -> HashMap<String, WorldAmbient> {
    catalog
      .ambients
      .iter()
      .filter_map(|shared| catalog.find_level_ambient(level, &shared.name))
      .map(|ambient| {
        let [least, most] = AmbientKey::get_effect_period(ambient, catalog.engine);

        (
          ambient.name.clone(),
          WorldAmbient {
            effects: ambient.get_list(AmbientKey::Effects).to_vec(),
            least_period: Self::to_milliseconds(least),
            most_period: Self::to_milliseconds(most),
          },
        )
      })
      .collect()
  }

  /// Every effect an ambient may play, `CEnvAmbient::create_effect`.
  pub fn to_world_effects(catalog: &EnvironmentCatalog) -> HashMap<String, WorldAmbientEffect> {
    catalog
      .ambient_effects
      .iter()
      .map(|effect| (effect.name.clone(), Self::to_world_effect(effect, catalog)))
      .collect()
  }

  /// The particles every effect of the catalog plays, which the level's particle surfaces must describe as well.
  pub fn list_particles(catalog: &EnvironmentCatalog) -> Vec<String> {
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
