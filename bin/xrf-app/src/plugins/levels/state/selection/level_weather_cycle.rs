use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, EnvironmentFinding, WeatherCycle, WeatherCycleKind, WeatherDescriptor};

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
  pub fn of(cycle: &WeatherCycle, catalog: &EnvironmentCatalog, engine: XrayEngine) -> Self {
    Self {
      file: cycle.file.clone(),
      findings: catalog
        .findings
        .iter()
        .filter(|finding| finding.file == cycle.file)
        .cloned()
        .collect(),
      keyframes: cycle
        .keyframes
        .iter()
        .filter(|keyframe| keyframe.time.is_some())
        .map(|keyframe| WeatherDescriptor::new(keyframe, engine))
        .collect(),
      kind: cycle.kind,
      name: cycle.name.clone(),
    }
  }
}
