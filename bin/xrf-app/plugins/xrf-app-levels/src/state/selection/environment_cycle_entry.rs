use serde::Serialize;
use xrf_environment::{EnvironmentCatalog, WeatherCycle, WeatherCycleKind};

/// One cycle or effect of a catalog, as a list shows it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCycleEntry {
  pub name: String,
  pub file: String,
  pub kind: WeatherCycleKind,
  pub keyframes: u32,
  /// Findings in its config.
  pub findings: u32,
}

impl EnvironmentCycleEntry {
  pub fn of(cycle: &WeatherCycle, catalog: &EnvironmentCatalog) -> Self {
    Self {
      file: cycle.file.clone(),
      findings: catalog
        .findings
        .iter()
        .filter(|finding| finding.file == cycle.file)
        .count() as u32,
      keyframes: cycle.keyframes.len() as u32,
      kind: cycle.kind,
      name: cycle.name.clone(),
    }
  }
}
