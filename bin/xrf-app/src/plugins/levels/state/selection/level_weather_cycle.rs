use serde::Serialize;
use xrf_environment::{EnvironmentCatalog, EnvironmentFinding, WeatherCycle, WeatherCycleKind, WeatherDescriptor};

use crate::plugins::levels::state::level_environment::LevelEnvironment;

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
}

impl LevelWeatherCycle {
  pub fn of(cycle: &WeatherCycle, catalog: &EnvironmentCatalog) -> Self {
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
      keyframes: LevelEnvironment::list_keyframes(cycle, catalog.engine),
    }
  }
}
