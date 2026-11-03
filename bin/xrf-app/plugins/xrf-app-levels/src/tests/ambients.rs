//! A level's ambients and their effects, as the world takes them.

use std::time::Duration;

use xrf_environment::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, EnvironmentCatalog, EnvironmentValue, LevelAmbients,
  SoundChannel, SoundChannelKey,
};
use xrf_world::{WorldAmbient, WorldAmbientEffect, WorldSound, WorldSoundChannel, WorldSoundInfo, WorldWindBlast};

use crate::tests::fixtures;
use crate::weather::LevelWorldAmbients;

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

/// Every sound found, a second long.
fn read_sound(reference: &str) -> WorldSound {
  WorldSound::new(reference, Some(WorldSoundInfo::default()))
}

fn new_channel(name: &str, values: Vec<(SoundChannelKey, EnvironmentValue)>) -> SoundChannel {
  let mut channel: SoundChannel = SoundChannel::new(name, "environment\\sound_channels.ltx");

  channel.values.extend(values);
  channel
}

// Call of Pripyat's periods are milliseconds; Shadow of Chernobyl's two `sound_period` seconds stand for both halves,
// and a `periodN` beside them overrides its half in seconds too; `sound_dist` is read nearest first.
#[test]
fn reads_a_channels_periods_and_distances_in_each_shape() {
  let mut catalog: EnvironmentCatalog = new_catalog();
  let sounds = || {
    (
      SoundChannelKey::Sounds,
      EnvironmentValue::List(vec![String::from("ambient\\wind")]),
    )
  };

  catalog.sound_channels = vec![
    new_channel(
      "cop",
      vec![
        sounds(),
        (SoundChannelKey::MinDistance, EnvironmentValue::Number(20.0)),
        (SoundChannelKey::MaxDistance, EnvironmentValue::Number(40.0)),
        (SoundChannelKey::Period0, EnvironmentValue::Integer(1000)),
        (SoundChannelKey::Period1, EnvironmentValue::Integer(2000)),
        (SoundChannelKey::Period2, EnvironmentValue::Integer(3000)),
        (SoundChannelKey::Period3, EnvironmentValue::Integer(4000)),
      ],
    ),
    new_channel(
      "soc",
      vec![
        sounds(),
        (SoundChannelKey::SoundDist, EnvironmentValue::Vector(vec![50.0, 10.0])),
        (SoundChannelKey::SoundPeriod, EnvironmentValue::Vector(vec![5.0, 8.0])),
        (SoundChannelKey::Period3, EnvironmentValue::Integer(9)),
      ],
    ),
  ];

  let channels = LevelWorldAmbients::to_world_channels(&catalog, &mut read_sound);

  assert_eq!(
    channels.get("cop"),
    Some(&WorldSoundChannel {
      sounds: vec![read_sound("ambient\\wind")],
      distance: [20.0, 40.0],
      period: [1000, 2000, 3000, 4000],
    })
  );
  assert_eq!(channels.get("soc").map(|it| it.distance), Some([10.0, 50.0]));
  assert_eq!(channels.get("soc").map(|it| it.period), Some([5000, 8000, 5000, 9000]));
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
    LevelWorldAmbients::to_world(&catalog, "zaton").get("day"),
    Some(&WorldAmbient {
      sound_channels: Vec::new(),
      effects: vec![String::from("fog")],
      least_period: Duration::from_secs(5),
      most_period: Duration::from_secs(6),
    })
  );
  assert_eq!(
    LevelWorldAmbients::to_world(&catalog, "jupiter")
      .get("day")
      .map(|it| it.effects.len()),
    Some(2)
  );
}

// `iFloor(life_time * 1000)`: the life in whole milliseconds; a blast only where its strength is written.
#[test]
fn reads_an_effects_life_and_wind_as_the_engine_does() {
  let effects = LevelWorldAmbients::to_world_effects(&new_catalog(), &mut read_sound);

  assert_eq!(
    effects.get("fog"),
    Some(&WorldAmbientEffect {
      particles: String::from("nature\\fog_stormy"),
      life_time: Duration::from_millis(10_000),
      offset: [0.0, 0.0, 10.0],
      wind_gust_factor: 0.015,
      wind_blast: WorldWindBlast::default(),
      sound: None,
    })
  );

  let blast: WorldWindBlast = effects.get("blast").map(|it| it.wind_blast).unwrap_or_default();

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

  let blast: WorldWindBlast = LevelWorldAmbients::to_world_effects(&catalog, &mut read_sound)
    .get("endless")
    .map(|it| it.wind_blast)
    .unwrap_or_default();

  assert_eq!(blast.in_time, Duration::from_secs(3600));
  assert_eq!(blast.out_time, Duration::ZERO);
}

#[test]
fn lists_every_effects_particles() {
  let mut particles: Vec<String> = LevelWorldAmbients::list_particles(&new_catalog());

  particles.sort();

  assert_eq!(
    particles,
    vec![String::from("nature\\fog_stormy"), String::from("nature\\vortex_01")]
  );
}
