//! The weather's ambient effects: which ambient a frame plays, and when its effects start, stop and go.

use std::collections::HashMap;
use std::time::Duration;

use glam::{Mat4, Vec3};
use xrf_engine_target::XrayEngine;
use xrf_particles::{
  ParticleEffect, ParticleEffectSprite, ParticleEngineRules, ParticleInstance, ParticleLibrary, ParticleObject,
  ParticleUpdateContext, ParticlesEffectsChunk, ParticlesFile, ParticlesGroupsChunk, ParticlesHeaderChunk,
};
use xrf_renderer::{
  ParticleEmitterProxy, RenderAmbient, RenderAmbientEffect, RenderAmbientReport, RenderAmbients, RenderLevelWeather,
  RenderSceneUpdate,
};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::level::ambient_frame::AmbientFrame;
use crate::level::level_ambient_effects::{AmbientSystems, LevelAmbientEffects};

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

/// The effects and the scene they post to: an emitter for each handle, its idle effect simulated as the renderer's
/// particles simulate it, and the emitters whose effect finished told back with the next update.
struct AmbientScene {
  library: ParticleLibrary,
  rules: ParticleEngineRules,
  effects: LevelAmbientEffects,
  allocator: ProxyAllocator<ParticleEmitterProxy>,
  emitters: HashMap<ProxyHandle<ParticleEmitterProxy>, SceneEmitter>,
  finished: Vec<ProxyHandle<ParticleEmitterProxy>>,
}

/// An emitter as the scene holds it: where it stands, its seed, and its idle effect with whether its end was told.
struct SceneEmitter {
  transform: Mat4,
  seed: i32,
  object: Option<(ParticleObject, bool)>,
}

impl AmbientScene {
  fn new(engine: XrayEngine, particles: &[&str]) -> Self {
    Self {
      library: library(particles),
      rules: ParticleEngineRules::new(engine, ParticleEngineRules::DEFAULT_UPDATE_COEFFICIENT),
      effects: LevelAmbientEffects::default(),
      allocator: ProxyAllocator::new(),
      emitters: HashMap::new(),
      finished: Vec::new(),
    }
  }

  /// Updates the effects at a moment, applies their posts, then steps what plays as the particles do.
  fn update(&mut self, frame: AmbientFrame<'_>, is_indoors: bool, now: u64) {
    let mut updates: Vec<RenderSceneUpdate> = Vec::new();
    let finished: Vec<ProxyHandle<ParticleEmitterProxy>> = std::mem::take(&mut self.finished);

    self.effects.update(
      &mut updates,
      AmbientSystems {
        library: &self.library,
        engine: self.rules.get_engine(),
        emitters: &mut self.allocator,
        finished: &finished,
      },
      Some(frame),
      (OUTDOORS, is_indoors),
      now,
    );

    let context: ParticleUpdateContext = ParticleUpdateContext {
      library: &self.library,
      rules: &self.rules,
      collider: None,
    };

    for update in updates {
      match update {
        RenderSceneUpdate::AddEmitter {
          handle,
          transform,
          seed,
        } => {
          self.emitters.insert(
            handle,
            SceneEmitter {
              transform,
              seed,
              object: None,
            },
          );
        }
        RenderSceneUpdate::PlayEffect { handle, name, .. } => {
          let emitter: &mut SceneEmitter = self.emitters.get_mut(&handle).expect("an emitter added");

          if let Some(instance) = self.library.create(&name, emitter.seed) {
            let mut object: ParticleObject = ParticleObject::new(instance);

            object.update_parent(&emitter.transform, Vec3::ZERO);
            object.play(now, &context);
            emitter.object = Some((object, false));
          }
        }
        RenderSceneUpdate::StopEffect {
          handle, is_deferred, ..
        } => {
          let emitter: &mut SceneEmitter = self.emitters.get_mut(&handle).expect("an emitter added");

          match &mut emitter.object {
            Some((object, _)) if is_deferred => object.stop(true),
            _ => emitter.object = None,
          }
        }
        RenderSceneUpdate::RemoveEmitter(handle) => {
          self.emitters.remove(&handle);
        }
        _ => panic!("The ambient effects post only to emitters"),
      }
    }

    for (handle, emitter) in &mut self.emitters {
      if let Some((object, is_told)) = &mut emitter.object {
        object.advance(now, OUTDOORS, true, &context);

        if !*is_told && !object.is_playing() {
          *is_told = true;
          self.finished.push(*handle);
        }
      }
    }
  }

  /// The effect playing in the scene, if any.
  fn get_playing(&self) -> Option<&ParticleObject> {
    self
      .emitters
      .values()
      .find_map(|emitter| emitter.object.as_ref().map(|(object, _)| object))
  }
}

/// The milliseconds at which effects start over a span, and what each played.
fn list_starts(engine: XrayEngine, names: &[&str], span: u64) -> Vec<(u64, String)> {
  let particles: Vec<String> = names.iter().map(|name| format!("nature\\{name}")).collect();
  let mut scene: AmbientScene = AmbientScene::new(engine, &particles.iter().map(String::as_str).collect::<Vec<_>>());
  let (weather, ambients) = (weather(names), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };
  let mut starts: Vec<(u64, String)> = Vec::new();
  let mut was_playing: bool = false;

  for now in (1..span).step_by(50) {
    scene.update(frame, false, now);

    let playing: Option<&ParticleObject> = scene.get_playing().filter(|it| !it.is_stopping());

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
  let mut scene: AmbientScene = AmbientScene::new(XrayEngine::Vanilla, &["nature\\fog"]);
  let (weather, ambients) = (weather(&["fog"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };

  scene.update(frame, false, 1);
  scene.update(frame, false, 900);
  assert!(scene.get_playing().is_some_and(|it| !it.is_stopping()));

  scene.update(frame, false, 1_001);
  assert!(scene.get_playing().is_none_or(|it| it.is_stopping()));

  scene.update(frame, false, 1_100);
  assert!(scene.get_playing().is_none());
}

#[test]
fn indoors_no_effect_starts_and_one_playing_stops() {
  let mut scene: AmbientScene = AmbientScene::new(XrayEngine::Vanilla, &["nature\\fog"]);
  let (weather, ambients) = (weather(&["fog"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };

  scene.update(frame, true, 1);
  assert!(scene.get_playing().is_none());

  let mut going_in: AmbientScene = AmbientScene::new(XrayEngine::Vanilla, &["nature\\fog"]);

  going_in.update(frame, false, 1);
  going_in.update(frame, true, 100);
  assert!(going_in.get_playing().is_none_or(|it| it.is_stopping()));
}

#[test]
fn an_ambient_without_effects_plays_none() {
  let mut scene: AmbientScene = AmbientScene::new(XrayEngine::Vanilla, &[]);
  let (weather, ambients) = (weather(&[]), day());

  scene.update(
    AmbientFrame {
      ambients: &ambients,
      level: &weather,
    },
    false,
    1,
  );
  assert!(scene.get_playing().is_none());
}

#[test]
fn playing_one_now_ends_the_one_playing_and_starts_without_waiting() {
  let mut scene: AmbientScene = AmbientScene::new(XrayEngine::Vanilla, &["nature\\fog", "nature\\leaves"]);
  let (weather, ambients) = (weather(&["fog", "leaves"]), day());
  let frame: AmbientFrame = AmbientFrame {
    ambients: &ambients,
    level: &weather,
  };

  scene.update(frame, false, 1);

  let first: RenderAmbientReport = scene.effects.report(500, false);

  assert!(
    first
      .effect
      .as_ref()
      .is_some_and(|it| (it.remaining - 0.501).abs() < 1e-3)
  );
  assert!(first.wait >= 9.5);

  scene.effects.play_now();
  scene.update(frame, false, 600);

  let second: RenderAmbientReport = scene.effects.report(600, false);

  assert!(
    second
      .effect
      .as_ref()
      .is_some_and(|it| (it.remaining - 1.0).abs() < 1e-3)
  );
  assert!(second.wait >= 10.0);
  assert_eq!(second.played, first.played + 1);

  // Indoors it is ended and none starts.
  scene.effects.play_now();
  scene.update(frame, true, 700);
  assert!(scene.get_playing().is_none());
  assert!(scene.effects.report(700, true).is_indoors);
}
