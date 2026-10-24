//! A level's ambients and their effects, as the renderer takes them.

use std::time::Duration;

use xrf_environment::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, EnvironmentCatalog, EnvironmentValue, LevelAmbients,
};
use xrf_renderer::{RenderAmbient, RenderAmbientEffect, RenderWindBlast};

use crate::render::LevelRenderAmbients;
use crate::tests::fixtures;

fn new_ambient(name: &str, effects: &[&str], period: [f32; 2]) -> Ambient {
  let mut ambient: Ambient = Ambient::new(name, "environment\\ambients.ltx");

  ambient.values.extend([
    (
      AmbientKey::Effects,
      EnvironmentValue::List(effects.iter().map(|it| (*it).to_owned()).collect()),
    ),
    (AmbientKey::MinEffectPeriod, EnvironmentValue::Number(period[0])),
    (AmbientKey::MaxEffectPeriod, EnvironmentValue::Number(period[1])),
  ]);

  ambient
}

fn new_effect(name: &str, particles: &str, values: &[(AmbientEffectKey, f32)]) -> AmbientEffect {
  let mut effect: AmbientEffect = AmbientEffect::new(name, "environment\\effects.ltx");

  effect.values.extend([
    (
      AmbientEffectKey::Particles,
      EnvironmentValue::Text(particles.to_owned()),
    ),
    (AmbientEffectKey::Offset, EnvironmentValue::Vector(vec![0.0, 0.0, 10.0])),
  ]);
  effect.values.extend(
    values
      .iter()
      .map(|(key, value)| (*key, EnvironmentValue::Number(*value))),
  );

  effect
}

fn new_catalog() -> EnvironmentCatalog {
  EnvironmentCatalog {
    ambient_effects: vec![
      new_effect(
        "fog",
        "nature\\fog_stormy",
        &[
          (AmbientEffectKey::LifeTime, 10.0005),
          (AmbientEffectKey::WindGustFactor, 0.015),
        ],
      ),
      new_effect(
        "blast",
        "nature\\vortex_01",
        &[
          (AmbientEffectKey::LifeTime, 5.0),
          (AmbientEffectKey::WindBlastStrength, 2.0),
          (AmbientEffectKey::WindBlastLongitude, 90.0),
          (AmbientEffectKey::WindBlastInTime, 0.5),
          (AmbientEffectKey::WindBlastOutTime, 1.5),
        ],
      ),
    ],
    ambients: vec![new_ambient("day", &["fog", "blast"], [30.0, 60.0])],
    level_ambients: vec![LevelAmbients {
      level: String::from("zaton"),
      file: String::from("environment\\ambients\\zaton.ltx"),
      ambients: vec![new_ambient("day", &["fog"], [5.0, 6.0])],
    }],
    ..fixtures::new_catalog()
  }
}

#[test]
fn takes_a_levels_own_ambient_over_the_shared_one() {
  let catalog: EnvironmentCatalog = new_catalog();

  assert_eq!(
    LevelRenderAmbients::to_render(&catalog, "zaton").get("day"),
    Some(&RenderAmbient {
      effects: vec![String::from("fog")],
      period: (Duration::from_secs(5), Duration::from_secs(6)),
    })
  );
  assert_eq!(
    LevelRenderAmbients::to_render(&catalog, "jupiter")
      .get("day")
      .map(|it| it.effects.len()),
    Some(2)
  );
}

// `iFloor(life_time * 1000)`: the life in whole milliseconds; a blast only where its strength is written.
#[test]
fn reads_an_effects_life_and_wind_as_the_engine_does() {
  let effects = LevelRenderAmbients::to_render_effects(&new_catalog());

  assert_eq!(
    effects.get("fog"),
    Some(&RenderAmbientEffect {
      particles: String::from("nature\\fog_stormy"),
      life_time: Duration::from_millis(10_000),
      offset: [0.0, 0.0, 10.0],
      wind_gust_factor: 0.015,
      wind_blast: RenderWindBlast::default(),
    })
  );

  let blast: RenderWindBlast = effects.get("blast").map(|it| it.wind_blast).unwrap_or_default();

  assert_eq!(blast.strength, 2.0);
  assert!((blast.longitude - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
  assert_eq!(blast.in_time, Duration::from_millis(500));
  assert_eq!(blast.out_time, Duration::from_millis(1500));
}

#[test]
fn reads_an_unbounded_wind_blast_ramp_as_an_hour() {
  let mut catalog: EnvironmentCatalog = new_catalog();

  catalog.ambient_effects.push(new_effect(
    "endless",
    "nature\\vortex_01",
    &[
      (AmbientEffectKey::WindBlastStrength, 1.0),
      (AmbientEffectKey::WindBlastInTime, f32::INFINITY),
      (AmbientEffectKey::WindBlastOutTime, f32::NAN),
    ],
  ));

  let blast: RenderWindBlast = LevelRenderAmbients::to_render_effects(&catalog)
    .get("endless")
    .map(|it| it.wind_blast)
    .unwrap_or_default();

  assert_eq!(blast.in_time, Duration::from_secs(3600));
  assert_eq!(blast.out_time, Duration::ZERO);
}

#[test]
fn lists_every_effects_particles() {
  let mut particles: Vec<String> = LevelRenderAmbients::list_particles(&new_catalog());

  particles.sort();

  assert_eq!(
    particles,
    vec![String::from("nature\\fog_stormy"), String::from("nature\\vortex_01")]
  );
}
