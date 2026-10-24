use serde::Serialize;
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
  /// `cycle` as `catalog`'s engine loads it, with the findings of its config.
  pub fn of(cycle: &WeatherCycle, catalog: &EnvironmentCatalog) -> Self {
    Self {
      file: cycle.file.clone(),
      findings: catalog.list_findings_of(&cycle.file).cloned().collect(),
      kind: cycle.kind,
      name: cycle.name.clone(),
      keyframes: cycle.list_descriptors(catalog.engine),
    }
  }
}
