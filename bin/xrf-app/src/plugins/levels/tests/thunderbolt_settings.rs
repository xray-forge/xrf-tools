//! Where bolts strike, as `CEffect_Thunderbolt`'s constructor loads it.

use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentValue, ThunderboltSettings, ThunderboltSettingsKey};

use crate::plugins::levels::state::selection::level_thunderbolt_settings::LevelThunderboltSettings;

fn new_settings(altitude: &[f32], min_distance: f32, second: f32) -> ThunderboltSettings {
  let mut settings: ThunderboltSettings = ThunderboltSettings::new("environment", "environment\\environment.ltx");

  settings.values.extend([
    (
      ThunderboltSettingsKey::Altitude,
      EnvironmentValue::Vector(altitude.to_vec()),
    ),
    (ThunderboltSettingsKey::DeltaLongitude, EnvironmentValue::Number(30.0)),
    (
      ThunderboltSettingsKey::MinDistFactor,
      EnvironmentValue::Number(min_distance),
    ),
    (ThunderboltSettingsKey::Tilt, EnvironmentValue::Number(17.0)),
    (
      ThunderboltSettingsKey::SecondPropability,
      EnvironmentValue::Number(second),
    ),
    (ThunderboltSettingsKey::SkyColor, EnvironmentValue::Number(0.1)),
    (ThunderboltSettingsKey::SunColor, EnvironmentValue::Number(0.9)),
    (ThunderboltSettingsKey::FogColor, EnvironmentValue::Number(0.1)),
  ]);

  settings
}

// Vanilla writes `altitude = 20`, which `try_read` of a pair fails on and reads as both bounds.
#[test]
fn reads_one_altitude_as_both_bounds_in_radians() {
  let settings = LevelThunderboltSettings::of(&new_settings(&[20.0], 0.94, 0.5), XrayEngine::Vanilla);

  assert_eq!(settings.altitude, [20f32.to_radians(), 20f32.to_radians()]);
  assert_eq!(settings.delta_longitude, 30f32.to_radians());
  assert_eq!(settings.tilt, 17f32.to_radians());
  assert_eq!(settings.min_distance, 0.94);
  assert_eq!(settings.sun_color, 0.9);
}

#[test]
fn reads_an_altitude_pair_and_clamps_as_the_engine_does() {
  let settings = LevelThunderboltSettings::of(&new_settings(&[10.0, 25.0], 0.99, 1.5), XrayEngine::Extended);

  assert_eq!(settings.altitude, [10f32.to_radians(), 25f32.to_radians()]);
  assert_eq!(settings.min_distance, LevelThunderboltSettings::MAX_DISTANCE);
  assert_eq!(settings.second_probability, 1.0);
}
