use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_environment::{ThunderboltSettings, ThunderboltSettingsKey};

/// Where bolts strike and how far they light the scene, as `CEffect_Thunderbolt` loads them: angles in radians.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderboltSettings {
  /// Above the horizon, the least and the most.
  pub altitude: [f32; 2],
  /// Either way of the heading opposite the sun.
  pub delta_longitude: f32,
  /// The nearest a bolt strikes, of the far plane.
  pub min_distance: f32,
  /// The most a bolt leans off the vertical.
  pub tilt: f32,
  /// The chance a strike is followed at once by another, clamped to a unit.
  pub second_probability: f32,
  /// How much of the strike's colour each is lit by.
  pub sky_color: f32,
  pub sun_color: f32,
  pub fog_color: f32,
}

impl LevelThunderboltSettings {
  /// `CEffect_Thunderbolt::MAX_DIST_FACTOR`, which the nearest distance never passes.
  pub const MAX_DISTANCE: f32 = 0.95;

  /// The settings once loaded: an altitude of one number is both bounds.
  pub fn of(settings: &ThunderboltSettings, engine: XrayEngine) -> Self {
    let altitude: &[f32] = settings
      .get(ThunderboltSettingsKey::Altitude)
      .and_then(|value| value.as_vector())
      .unwrap_or(&[]);
    let least: f32 = altitude.first().copied().unwrap_or(0.0);
    let number = |key: ThunderboltSettingsKey| settings.get_number(key, engine);

    Self {
      altitude: [least, altitude.get(1).copied().unwrap_or(least)].map(f32::to_radians),
      delta_longitude: number(ThunderboltSettingsKey::DeltaLongitude).to_radians(),
      fog_color: number(ThunderboltSettingsKey::FogColor),
      min_distance: number(ThunderboltSettingsKey::MinDistFactor).min(Self::MAX_DISTANCE),
      second_probability: number(ThunderboltSettingsKey::SecondPropability).clamp(0.0, 1.0),
      sky_color: number(ThunderboltSettingsKey::SkyColor),
      sun_color: number(ThunderboltSettingsKey::SunColor),
      tilt: number(ThunderboltSettingsKey::Tilt).to_radians(),
    }
  }
}
