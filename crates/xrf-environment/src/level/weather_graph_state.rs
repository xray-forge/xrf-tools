use serde::Serialize;

/// One state of a weather graph and how likely the script is to pick it each hour.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherGraphState {
  /// `clear`, `rain`: the script plays `default_<state>`.
  pub state: String,
  /// The state's weight against the graph's others.
  pub weight: f32,
}
