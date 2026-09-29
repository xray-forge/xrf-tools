use serde::{Deserialize, Serialize};

/// Which list the engine keeps a cycle in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WeatherCycleKind {
  /// A day, under `environment\weathers`, which `set_weather` plays: `load_weathers`.
  Cycle,
  /// An effect, under `environment\weather_effects`, which `start_weather_fx` plays over a cycle, bracketed by the
  /// engine with keyframes of its own at midnight either end: `load_weather_effects`.
  Effect,
}

impl WeatherCycleKind {
  /// How a finding names a keyframe of this kind.
  pub fn get_subject(self) -> &'static str {
    match self {
      Self::Cycle => "Weather",
      Self::Effect => "Weather effect",
    }
  }
}
