use serde::Serialize;

use crate::level::weather_graph_state::WeatherGraphState;

/// A `dynamic_weather_graphs.ltx` graph, which `level_weathers.script` walks each game hour: a state picked by weight,
/// played as the cycle `default_<state>`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherGraph {
  pub name: String,
  pub states: Vec<WeatherGraphState>,
}

impl WeatherGraph {
  /// The cycle the script plays for a state of a graph.
  pub fn cycle_of(state: &str) -> String {
    format!("default_{state}")
  }
}
