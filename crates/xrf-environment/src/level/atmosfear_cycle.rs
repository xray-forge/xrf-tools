use serde::Serialize;

/// One of Atmosfear's weather states on Anomaly, `[weather_cycles]` in `dynamic_weather_graphs.ltx`: the state and
/// the cycles `[cycle_<state>]` lists for it, one of which its script plays.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtmosfearCycle {
  /// `clear`, `storm`.
  pub state: String,
  /// Cycles under `environment\weathers`, in the order listed.
  pub presets: Vec<String>,
}
