use serde::{Deserialize, Serialize};
use xrf_environment::WeatherDescriptor;

/// What a level viewport's weather plays.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorldWeatherPlay {
  /// Nothing: the level is lit by noon of `default_clear`, standing still.
  None,
  /// A cycle of the level's game, by name, read through the level's source.
  Cycle { name: String },
  /// One keyframe set by hand, played as a cycle of one: its sun stands by its own angles on either engine.
  Keyframe { keyframe: Box<WeatherDescriptor> },
}
