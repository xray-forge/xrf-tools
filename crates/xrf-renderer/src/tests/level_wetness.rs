use glam::Vec3;

use crate::contract::render_rain_settings::RenderRainSettings;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::scene::level::level_wetness::LevelWetness;

fn raining(density: f32) -> RenderLighting {
  RenderLighting {
    rain: Some(RenderRainfall {
      color: Vec3::ONE,
      density,
    }),
    ..RenderLighting::default()
  }
}

// The level holds what its rain did, dry and empty before any, and moving it on again at the clock it stands at, as a
// view asking a second time would, changes nothing whatever that view's rain.
#[test]
fn keeps_the_rain_it_was_moved_on_by() {
  let mut level: LevelWetness = LevelWetness::default();
  let settings: RenderRainSettings = RenderRainSettings::default();

  assert_eq!(level.get_wetness().get(), 0.0);
  assert_eq!(level.get_puddles(), (0.0, 0.0, 0.0));

  level.advance(&raining(1.0), &settings, 0.0);
  level.advance(&raining(1.0), &settings, 60.0);

  let wetness: f32 = level.get_wetness().get();
  let puddles: (f32, f32, f32) = level.get_puddles();

  assert!(wetness > 0.0 && puddles.0 > 0.0);

  level.advance(&RenderLighting::default(), &settings, 60.0);

  assert_eq!(level.get_wetness().get(), wetness);
  assert_eq!(level.get_puddles(), puddles);
}

// A rain's wetting at a faster pace comes to what the game's own pace reaches over as many times as long.
#[test]
fn a_faster_pace_wets_and_fills_as_a_longer_rain() {
  let settings: RenderRainSettings = RenderRainSettings::default();
  let paced: RenderRainSettings = RenderRainSettings {
    pace: 100.0,
    ..settings
  };
  let (mut slow, mut fast): (LevelWetness, LevelWetness) = (LevelWetness::default(), LevelWetness::default());

  for (level, settings, span) in [(&mut slow, &settings, 100.0), (&mut fast, &paced, 1.0)] {
    level.advance(&raining(0.6), settings, 0.0);
    level.advance(&raining(0.6), settings, span);
  }

  assert!((slow.get_wetness().get() - fast.get_wetness().get()).abs() < 1e-5);
  assert!((slow.get_puddles().0 - fast.get_puddles().0).abs() < 1e-5);
  assert!(fast.get_puddles().0 > 0.0);
}
