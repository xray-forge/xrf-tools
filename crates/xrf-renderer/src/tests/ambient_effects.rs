//! The weather's ambient effects: which ambient a frame plays, and when its effects start, stop and go.

use std::collections::HashMap;
use std::time::Duration;

use glam::Vec3;
use xrf_engine_target::XrayEngine;
use xrf_particles::{
  ParticleEffect, ParticleEffectSprite, ParticleEngineRules, ParticleInstance, ParticleLibrary, ParticleUpdateContext,
  ParticlesEffectsChunk, ParticlesFile, ParticlesGroupsChunk, ParticlesHeaderChunk,
};

use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::host::render_ambient::RenderAmbient;
use crate::host::render_ambient_effect::RenderAmbientEffect;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_ambients::RenderAmbients;
use crate::scene::level::ambient_frame::AmbientFrame;
use crate::scene::level::level_ambient_effects::LevelAmbientEffects;

const OUTDOORS: Vec3 = Vec3::new(10.0, 2.0, -5.0);

/// A looping effect that emits nothing, so a deferred stop ends it on its next update.
fn effect(name: &str) -> ParticleEffect {
  ParticleEffect {
    version: 1,
    name: String::from(name),
    max_particles: 1,
    actions: Vec::new(),
    flags: 1,
    frame: None,
    sprite: ParticleEffectSprite {
      shader_name: String::from("particles\\add"),
      texture_name: String::from("pfx\\pfx_fog"),
    },
    time_limit: None,
    collision: None,
    velocity_scale: None,
    description: None,
    rotation: None,
    editor_data: None,
  }
}

fn library(names: &[&str]) -> ParticleLibrary {
  ParticleLibrary::from(ParticlesFile {
    header: ParticlesHeaderChunk { version: 1 },
    effects: ParticlesEffectsChunk {
      effects: names.iter().map(|name| effect(name)).collect(),
    },
    groups: ParticlesGroupsChunk { groups: Vec::new() },
  })
}

/// A level whose `day` ambient plays the named effects, each a second long, every ten to twenty seconds.
fn weather(effects: &[&str]) -> RenderLevelWeather {
  RenderLevelWeather {
    ambients: HashMap::from([(
      String::from("day"),
      RenderAmbient {
        effects: effects.iter().map(|it| String::from(*it)).collect(),
        period: (Duration::from_secs(10), Duration::from_secs(20)),
      },
    )]),
    ambient_effects: effects
      .iter()
      .map(|name| {
        (
          String::from(*name),
          RenderAmbientEffect {
            particles: format!("nature\\{name}"),
            life_time: Duration::from_secs(1),
            offset: [0.0, 5.0, 0.0],
            ..RenderAmbientEffect::default()
          },
        )
      })
      .collect(),
    ..RenderLevelWeather::default()
  }
}

fn day() -> RenderAmbients {
  RenderAmbients {
    names: [Some(String::from("day")), Some(String::from("day"))],
    weight: 0.3,
  }
}

/// Updates the effects at a moment, then steps what plays as the particles do.
fn update(
  effects: &mut LevelAmbientEffects,
  frame: AmbientFrame<'_>,
  is_indoors: bool,
  now: u64,
  context: &ParticleUpdateContext,
) {
  effects.update(Some(frame), (OUTDOORS, is_indoors), now, context);

  if let Some(object) = effects.get_playing_mut() {
    object.advance(now, OUTDOORS, true, context);
  }
}

/// The milliseconds at which effects start over a span, and what each played.
fn list_starts(engine: XrayEngine, names: &[&str], span: u64) -> Vec<(u64, String)> {
  let particles: Vec<String> = names.iter().map(|name| format!("nature\\{name}")).collect();
  let library: ParticleLibrary = library(&particles.iter().map(String::as_str).collect::<Vec<_>>());
  let rules: ParticleEngineRules = ParticleEngineRules::new(engine, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context: ParticleUpdateContext = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };
  let (weather, ambients) = (weather(names), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };
  let mut effects: LevelAmbientEffects = LevelAmbientEffects::default();
  let mut starts: Vec<(u64, String)> = Vec::new();
  let mut was_playing: bool = false;

  for now in (1..span).step_by(50) {
    update(&mut effects, frame, false, now, &context);

    let playing = effects.get_playing().filter(|it| !it.is_stopping());

    if let Some(playing) = playing
      && !was_playing
    {
      let name: String = match playing.get_instance() {
        ParticleInstance::Effect(effect) => effect.get_definition().name.clone(),
        ParticleInstance::Group(_) => String::new(),
      };

      starts.push((now, name));
    }

    was_playing = playing.is_some();
  }

  starts
}

#[test]
fn a_frame_plays_the_first_keyframes_ambient_while_the_chance_falls_short_of_the_way_left() {
  let ambients: RenderAmbients = RenderAmbients {
    names: [Some(String::from("dawn")), Some(String::from("day"))],
    weight: 0.25,
  };

  assert_eq!(ambients.pick(0.0), Some("dawn"));
  assert_eq!(ambients.pick(0.74), Some("dawn"));
  assert_eq!(ambients.pick(0.75), Some("day"));
  assert!(!ambients.is_settled());
}

#[test]
fn an_effect_starts_at_once_outdoors_then_waits_its_ambients_period() {
  let starts: Vec<(u64, String)> = list_starts(XrayEngine::Vanilla, &["fog"], 60_000);

  assert_eq!(starts[0], (1, String::from("nature\\fog")));
  assert!(starts.len() >= 3, "{starts:?}");

  for pair in starts.windows(2) {
    let wait: u64 = pair[1].0 - pair[0].0;

    assert!((10_000..=20_050).contains(&wait), "{starts:?}");
  }
}

#[test]
fn the_same_level_plays_the_same_effects_at_the_same_moments() {
  let names: [&str; 3] = ["fog", "leaves", "vortex"];

  assert_eq!(
    list_starts(XrayEngine::Extended, &names, 120_000),
    list_starts(XrayEngine::Extended, &names, 120_000)
  );
}

#[test]
fn an_effect_stops_once_its_life_is_up_and_goes_once_it_has_died() {
  let library: ParticleLibrary = library(&["nature\\fog"]);
  let rules: ParticleEngineRules =
    ParticleEngineRules::new(XrayEngine::Vanilla, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context: ParticleUpdateContext = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };
  let (weather, ambients) = (weather(&["fog"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };
  let mut effects: LevelAmbientEffects = LevelAmbientEffects::default();

  update(&mut effects, frame, false, 1, &context);
  update(&mut effects, frame, false, 900, &context);
  assert!(effects.get_playing().is_some_and(|it| !it.is_stopping()));

  update(&mut effects, frame, false, 1_001, &context);
  assert!(effects.get_playing().is_none_or(|it| it.is_stopping()));

  update(&mut effects, frame, false, 1_100, &context);
  assert!(effects.get_playing().is_none());
}

#[test]
fn indoors_no_effect_starts_and_one_playing_stops() {
  let library: ParticleLibrary = library(&["nature\\fog"]);
  let rules: ParticleEngineRules =
    ParticleEngineRules::new(XrayEngine::Vanilla, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context: ParticleUpdateContext = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };
  let (weather, ambients) = (weather(&["fog"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };
  let mut indoors: LevelAmbientEffects = LevelAmbientEffects::default();

  update(&mut indoors, frame, true, 1, &context);
  assert!(indoors.get_playing().is_none());

  let mut going_in: LevelAmbientEffects = LevelAmbientEffects::default();

  update(&mut going_in, frame, false, 1, &context);
  update(&mut going_in, frame, true, 100, &context);
  assert!(going_in.get_playing().is_none_or(|it| it.is_stopping()));
}

#[test]
fn an_ambient_without_effects_plays_none() {
  let library: ParticleLibrary = library(&[]);
  let rules: ParticleEngineRules =
    ParticleEngineRules::new(XrayEngine::Vanilla, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context: ParticleUpdateContext = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };
  let (weather, ambients) = (weather(&[]), day());
  let mut effects: LevelAmbientEffects = LevelAmbientEffects::default();

  update(
    &mut effects,
    AmbientFrame {
      ambients: &ambients,
      level: &weather,
    },
    false,
    1,
    &context,
  );
  assert!(effects.get_playing().is_none());
}

#[test]
fn playing_one_now_ends_the_one_playing_and_starts_without_waiting() {
  let library: ParticleLibrary = library(&["nature\\fog", "nature\\leaves"]);
  let rules: ParticleEngineRules =
    ParticleEngineRules::new(XrayEngine::Vanilla, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT);
  let context: ParticleUpdateContext = ParticleUpdateContext {
    library: &library,
    rules: &rules,
    collider: None,
  };
  let (weather, ambients) = (weather(&["fog", "leaves"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };
  let mut effects: LevelAmbientEffects = LevelAmbientEffects::default();

  update(&mut effects, frame, false, 1, &context);

  let first: RenderAmbientReport = effects.report(500, false);

  assert!(
    first
      .effect
      .as_ref()
      .is_some_and(|it| (it.remaining - 0.501).abs() < 1e-3)
  );
  assert!(first.wait >= 9.5);

  effects.play_now();
  update(&mut effects, frame, false, 600, &context);

  let second: RenderAmbientReport = effects.report(600, false);

  assert!(
    second
      .effect
      .as_ref()
      .is_some_and(|it| (it.remaining - 1.0).abs() < 1e-3)
  );
  assert!(second.wait >= 10.0);

  // Indoors it is ended and none starts.
  effects.play_now();
  update(&mut effects, frame, true, 700, &context);
  assert!(effects.get_playing().is_none());
  assert!(effects.report(700, true).is_indoors);
}
