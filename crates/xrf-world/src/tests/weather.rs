use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_environment::{WeatherDescriptor, WeatherMix, WeatherMixPoint, WeatherMixer, WeatherSunSource};
use xrf_material::XraySurfaceDraw;
use xrf_renderer::{
  RenderLevelWeather, RenderLighting, RenderThunder, RenderThunderSettings, RenderThunderbolt,
  RenderThunderboltGradient, RenderWeatherControl, RenderWeatherTransition,
};
use xrf_visual::{LightAnimatorDescription, LightAnimatorKey};

use crate::weather::played_weather::PlayedWeather;
use crate::weather::weather_fade::to_faded_lighting;
use crate::weather::weather_player::WeatherPlayer;
use crate::weather::weather_thunder::WeatherThunder;

const NOON: u32 = 12 * 3600;

/// A keyframe at a time of day, its sun of a colour straight down, everything else plain.
fn keyframe(time: u32, sun: f32) -> WeatherDescriptor {
  WeatherDescriptor {
    time,
    sky_texture: format!("sky\\sky_{time}"),
    sky_texture_env: format!("sky\\sky_{time}#small"),
    sky_color: [1.0; 3],
    sky_rotation: 0.0,
    clouds_texture: String::new(),
    clouds_color: [0.0; 4],
    clouds_rotation: 0.0,
    far_plane: 1000.0,
    fog_color: [0.5; 3],
    fog_density: 0.5,
    fog_distance: 900.0,
    rain_density: 0.0,
    rain_color: [0.5; 3],
    wind_velocity: 0.0,
    wind_direction: 0.0,
    hemi_color: [0.4, 0.4, 0.4, 1.0],
    sun_color: [sun; 3],
    ambient_color: [0.02; 3],
    ambient: None,
    sun: None,
    sun_direction: Some([0.0, -1.0, 0.0]),
    is_sun_fixed: false,
    sun_azimuth: 0.0,
    sun_shafts_intensity: 0.0,
    water_intensity: 1.0,
    tree_amplitude: 0.005,
    tree_speed: 1.0,
    tree_rotation: 10.0,
    tree_wave: [0.1, 0.01, 0.11],
    thunderbolt_collection: None,
    thunderbolt_period: 0.0,
    thunderbolt_duration: 0.0,
    hemi_vibrance: 1.0,
    hemi_contrast: 1.0,
    wet_surface_factor: 0.0,
    volumetric_intensity_factor: 1.0,
    volumetric_distance_factor: 1.0,
    bloom_threshold: 3.5,
    bloom_exposure: 3.0,
    bloom_sky_intensity: 0.6,
  }
}

/// Noon lit at a half, two in the afternoon lit whole.
fn afternoon() -> PlayedWeather {
  PlayedWeather::cycle(
    vec![keyframe(NOON, 0.5), keyframe(NOON + 7200, 1.0)],
    Arc::new(RenderLevelWeather::default()),
  )
}

fn sun_of(lighting: &RenderLighting) -> f32 {
  lighting.sun_color.x
}

#[test]
fn plays_a_cycle_from_a_seek_and_runs_its_clock() {
  let start: Instant = Instant::now();
  let mut player: WeatherPlayer = WeatherPlayer::default();

  player.take(Some(afternoon()), RenderWeatherTransition::Cut);
  // A second past noon: exactly on a keyframe, `SelectEnvs` blends towards it from the one before.
  player.seek((NOON + 1) as f32);

  let lit: RenderLighting = player.advance(start, [0.0; 3], false, |_| true).unwrap();

  assert!((sun_of(&lit) - 0.5).abs() < 1e-3);
  assert_eq!(player.report().unwrap().between, [NOON as f32, (NOON + 7200) as f32]);

  // An hour a second, one second on: halfway to two.
  player.set_control(RenderWeatherControl {
    factor: 3600.0,
    is_paused: false,
    is_dynamic_sun: false,
  });
  player.advance(start, [0.0; 3], false, |_| true);

  let later: RenderLighting = player
    .advance(start + Duration::from_secs(1), [0.0; 3], false, |_| true)
    .unwrap();

  assert!((sun_of(&later) - 0.75).abs() < 1e-3);
  assert!((player.report().unwrap().time - (NOON + 3601) as f32).abs() < 1e-2);
  // A paused clock lights nothing anew.
  player.set_control(RenderWeatherControl::default());
  player.advance(start + Duration::from_secs(1), [0.0; 3], false, |_| true);
  assert_eq!(
    player.advance(start + Duration::from_secs(2), [0.0; 3], false, |_| true),
    None
  );
}

#[test]
fn fades_into_another_weather_once_its_skies_are_up() {
  let start: Instant = Instant::now();
  let mut player: WeatherPlayer = WeatherPlayer::default();

  player.take(Some(afternoon()), RenderWeatherTransition::Cut);
  player.seek(NOON as f32);
  player.advance(start, [0.0; 3], false, |_| true);

  let bright: PlayedWeather = PlayedWeather::cycle(
    vec![keyframe(0, 1.5), keyframe(NOON, 1.5)],
    Arc::new(RenderLevelWeather::default()),
  );

  player.take(Some(bright), RenderWeatherTransition::Fade);

  // Its skies still going up, it waits on what was shown.
  let waiting: RenderLighting = player.advance(start, [0.0; 3], false, |_| false).unwrap();

  assert!((sun_of(&waiting) - 0.5).abs() < 1e-6);

  let started: RenderLighting = player
    .advance(start + Duration::from_millis(100), [0.0; 3], false, |_| true)
    .unwrap();
  let halfway: RenderLighting = player
    .advance(start + Duration::from_millis(850), [0.0; 3], false, |_| true)
    .unwrap();
  let done: RenderLighting = player
    .advance(start + Duration::from_millis(1700), [0.0; 3], false, |_| true)
    .unwrap();

  assert!((sun_of(&started) - 0.5).abs() < 1e-6);
  assert!((sun_of(&halfway) - 1.0).abs() < 1e-3);
  assert!((sun_of(&done) - 1.5).abs() < 1e-6);
}

#[test]
fn lays_an_effect_over_the_cycle_and_gives_the_cycle_back() {
  let start: Instant = Instant::now();
  let effect: Vec<WeatherDescriptor> = vec![keyframe(0, 2.0), keyframe(60, 2.0)];
  let level: RenderLevelWeather = RenderLevelWeather {
    effects: HashMap::from([("fx".to_owned(), effect)]),
    ..Default::default()
  };
  let mut player: WeatherPlayer = WeatherPlayer::default();

  player.take(
    Some(PlayedWeather::cycle(
      vec![keyframe(0, 0.5), keyframe(NOON, 0.5)],
      Arc::new(level),
    )),
    RenderWeatherTransition::Cut,
  );
  player.seek(1000.0);
  player.advance(start, [0.0; 3], false, |_| true);
  player.play_effect(Some("fx"));
  player.advance(start, [0.0; 3], false, |_| true);

  let playing = player.report().unwrap().effect.unwrap();

  assert_eq!(playing.name, "fx");
  // A lead-in, its own minute, and a lead-in back, at twelve game seconds a real one.
  assert!((playing.remaining - (60.0 + 2.0 * 5.0 * 12.0)).abs() < 1e-3);

  player.set_control(RenderWeatherControl {
    factor: 1000.0,
    is_paused: false,
    is_dynamic_sun: false,
  });

  for second in 1..=2 {
    player.advance(start + Duration::from_secs(second), [0.0; 3], false, |_| true);
  }

  assert_eq!(player.report().unwrap().effect, None);
}

#[test]
fn stands_a_keyframe_set_by_hand_by_its_own_angles() {
  let control: RenderWeatherControl = RenderWeatherControl {
    is_dynamic_sun: true,
    ..Default::default()
  };
  let manual: PlayedWeather = PlayedWeather::keyframe(keyframe(NOON, 1.0), Arc::new(RenderLevelWeather::default()));

  assert_eq!(manual.get_sun(&control), WeatherSunSource::Authored);
  assert_eq!(afternoon().get_sun(&control), WeatherSunSource::Dynamic);
}

#[test]
fn walks_the_skies_from_one_pair_to_the_other_in_thirds() {
  let sky = |first: &str, second: &str, blend: f32| {
    let mut lighting: RenderLighting = RenderLighting::default();

    lighting.sky.textures = [Some(first.to_owned()), Some(second.to_owned())];
    lighting.sky.blend = blend;
    lighting
  };
  let (from, to) = (sky("a", "b", 0.8), sky("c", "d", 0.2));
  let at = |t: f32| {
    let faded: RenderLighting = to_faded_lighting(&from, &to, t);

    (faded.sky.textures.map(Option::unwrap), faded.sky.blend)
  };

  // Leaving the old pair on its heavier half, then from that half to the new pair's heavier, then to its own blend.
  assert_eq!(at(0.0), (["a".into(), "b".into()], 0.8));
  assert_eq!(at(0.5).0, ["b".to_owned(), "c".to_owned()]);
  assert!((at(0.5).1 - 0.5).abs() < 1e-5);
  assert_eq!(at(1.0), (["c".into(), "d".into()], 0.2));
}

fn thunder() -> RenderThunder {
  let gradient: RenderThunderboltGradient = RenderThunderboltGradient {
    opacity: 1.0,
    radius: [0.1, 0.2],
    texture: "fx\\fx_glow".to_owned(),
    draw: XraySurfaceDraw::Added {
      reference: 0,
      is_weighted: false,
    },
  };

  RenderThunder {
    settings: Some(RenderThunderSettings {
      altitude: [0.3, 0.6],
      delta_longitude: 0.5,
      min_distance: 0.5,
      tilt: 0.1,
      second_probability: 0.0,
      sky_color: 1.0,
      sun_color: 1.0,
      fog_color: 1.0,
    }),
    collections: HashMap::from([("storm".to_owned(), vec!["bolt".to_owned()])]),
    bolts: HashMap::from([(
      "bolt".to_owned(),
      RenderThunderbolt {
        model: None,
        color: Some(0),
        top: gradient.clone(),
        center: gradient,
      },
    )]),
    models: Vec::new(),
    animators: vec![LightAnimatorDescription {
      fps: 10.0,
      frame_count: 10,
      keys: vec![LightAnimatorKey {
        frame: 0,
        color: [255.0, 128.0, 0.0],
      }],
    }],
  }
}

fn storm_mix() -> WeatherMix {
  let mut stormy: WeatherDescriptor = keyframe(NOON, 1.0);

  stormy.thunderbolt_collection = Some("storm".to_owned());
  stormy.thunderbolt_period = 2.0;
  stormy.thunderbolt_duration = 0.5;

  WeatherMixer {
    engine: Default::default(),
    sun: WeatherSunSource::Authored,
    modifiers: &[],
  }
  .mix(
    &[stormy],
    WeatherMixPoint {
      time: NOON as f32,
      view: [0.0; 3],
    },
  )
  .unwrap()
}

#[test]
fn strikes_a_bolt_of_the_collection_each_period_and_lights_by_its_colour() {
  let (thunder, mix) = (thunder(), storm_mix());
  let mut strikes: WeatherThunder = WeatherThunder::new(7);
  let mut struck: Vec<f32> = Vec::new();

  // A period half again at most after the weather names the collection, then a strike lasting about half a second.
  for step in 0..200 {
    let now: f32 = step as f32 * 0.05;

    if let Some(flash) = strikes.advance(&thunder, &mix, true, now, Vec3::ZERO) {
      assert_eq!(flash.strike.bolt, "bolt");
      assert!(flash.color.distance(Vec3::new(1.0, 128.0 / 255.0, 0.0)) < 1e-5);
      // It strikes beyond the view, from above the ground.
      assert!(flash.strike.position.y > 0.0);
      struck.push(now);
    }
  }

  assert!(!struck.is_empty());
  assert!(struck[0] >= 1.0 - 1e-3 && struck[0] <= 3.05);

  // Turned off, nothing strikes.
  let mut quiet: WeatherThunder = WeatherThunder::new(7);

  assert!((0..200).all(|step| {
    quiet
      .advance(&thunder, &mix, false, step as f32 * 0.05, Vec3::ZERO)
      .is_none()
  }));
}
