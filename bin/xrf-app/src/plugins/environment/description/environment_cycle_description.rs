use serde::Serialize;
use xrf_environment::{EnvironmentFinding, WeatherCycle};

/// One cycle or effect as authored, where each value came from, and what is wrong in its config.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCycleDescription {
  pub cycle: WeatherCycle,
  pub findings: Vec<EnvironmentFinding>,
}
