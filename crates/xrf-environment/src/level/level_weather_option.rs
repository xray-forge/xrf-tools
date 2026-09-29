use serde::Serialize;

/// One cycle a level can be lit under, and what in its weather picks it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWeatherOption {
  /// The cycle, under `environment\weathers`.
  pub cycle: String,
  /// The graph that plays it, or `atmosfear`; none for a cycle the level names itself.
  pub graph: Option<String>,
  /// The graph's or Atmosfear's state that plays it, `clear` or `rain`.
  pub state: Option<String>,
}
