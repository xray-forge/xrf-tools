use serde::{Deserialize, Serialize};
use xrf_environment::WeatherDescriptor;

use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_weather_effect_report::RenderWeatherEffectReport;

/// Where a viewport's weather stands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWeatherReport {
  /// Seconds since midnight.
  pub time: f32,
  /// The times of the two keyframes blended between, the effect's own while one plays.
  pub between: [f32; 2],
  /// How far from the first to the second.
  pub weight: f32,
  /// The effect playing, or none.
  pub effect: Option<RenderWeatherEffectReport>,
  /// How many of the level's modifiers reach the camera.
  pub modifiers: u32,
  /// What is mixed now as one keyframe, without the modifiers: what a keyframe set by hand starts from.
  pub current: Box<WeatherDescriptor>,
  /// The ambient effects near the camera, none until the level's particles are read.
  pub ambient: Option<RenderAmbientReport>,
}
