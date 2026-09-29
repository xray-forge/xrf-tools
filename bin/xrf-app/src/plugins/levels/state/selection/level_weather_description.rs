use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_environment::{LevelWeather, SunPosition};
use xrf_level::EnvModifier;

use crate::plugins::environment::description::environment_cycle_entry::EnvironmentCycleEntry;
use crate::plugins::levels::state::selection::level_rain::LevelRain;
use crate::plugins::levels::state::selection::level_thunderbolts::LevelThunderbolts;
use crate::plugins::levels::state::selection::level_weather_cycle::LevelWeatherCycle;
use crate::plugins::levels::state::selection::level_weather_texture::LevelWeatherTexture;
use crate::plugins::levels::state::selection::level_wet_surfaces::LevelWetSurfaces;

/// Everything a viewer plays the open level's weather from, as its engine loads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWeatherDescription {
  pub engine: XrayEngine,
  /// What the level's `weathers` resolves to.
  pub weather: LevelWeather,
  /// The cycles it resolves to that the game has, in the order it offers them.
  pub offered: Vec<LevelWeatherCycle>,
  /// Every cycle of the game, which a viewer offers after the level's own.
  pub cycles: Vec<EnvironmentCycleEntry>,
  /// Every weather effect, which the game plays over a cycle.
  pub effects: Vec<LevelWeatherCycle>,
  pub thunderbolts: LevelThunderbolts,
  /// Monolith's table of where the sun stands, midnight first; none on OpenXRay.
  pub sun_table: Option<Vec<SunPosition>>,
  /// The level's local overrides, `level.env_mod`; none where it has none.
  pub modifiers: Vec<EnvModifier>,
  pub rain: LevelRain,
  pub wet: LevelWetSurfaces,
  /// Every sky the cycles and effects name, by reference, for a hand-set keyframe to be given.
  pub skies: Vec<LevelWeatherTexture>,
  /// Every clouds texture they name.
  pub clouds: Vec<LevelWeatherTexture>,
}
