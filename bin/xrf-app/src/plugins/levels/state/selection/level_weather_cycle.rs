use std::collections::BTreeSet;

use serde::Serialize;
use xrf_environment::{EnvironmentCatalog, EnvironmentFinding, WeatherCycle, WeatherCycleKind, WeatherDescriptor};

use crate::plugins::levels::state::level_environment::LevelEnvironment;
use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

/// One cycle or effect as the engine loads it, which a viewer mixes, and what is wrong in its config.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWeatherCycle {
  pub name: String,
  pub file: String,
  pub kind: WeatherCycleKind,
  /// Sorted by time, a keyframe whose name the engine refuses left out.
  pub keyframes: Vec<WeatherDescriptor>,
  pub findings: Vec<EnvironmentFinding>,
  /// Every sky, irradiance cube and clouds texture its keyframes name, each once, as the level finds them.
  pub textures: Vec<LevelTextureReference>,
}

impl LevelWeatherCycle {
  pub fn of(cycle: &WeatherCycle, source: &LevelWeatherSource) -> Self {
    let catalog: &EnvironmentCatalog = source.catalog;
    let keyframes: Vec<WeatherDescriptor> = LevelEnvironment::list_keyframes(cycle, catalog.engine);
    // A keyframe without a sky names only the suffix of its irradiance cube, which nothing answers to.
    let references: BTreeSet<&str> = keyframes
      .iter()
      .flat_map(|keyframe| {
        [
          keyframe.sky_texture.as_str(),
          if keyframe.sky_texture.is_empty() {
            ""
          } else {
            keyframe.sky_texture_env.as_str()
          },
          keyframe.clouds_texture.as_str(),
        ]
      })
      .filter(|reference| !reference.is_empty())
      .collect();

    Self {
      file: cycle.file.clone(),
      findings: catalog
        .findings
        .iter()
        .filter(|finding| finding.file == cycle.file)
        .cloned()
        .collect(),
      kind: cycle.kind,
      name: cycle.name.clone(),
      textures: references
        .into_iter()
        .map(|reference| source.locate(reference))
        .collect(),
      keyframes,
    }
  }
}
