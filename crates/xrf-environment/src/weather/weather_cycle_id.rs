use serde::{Deserialize, Serialize};

use crate::weather::weather_cycle_kind::WeatherCycleKind;

/// Which cycle or effect: the list the engine keeps it in, and its name there.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherCycleId {
  pub kind: WeatherCycleKind,
  /// The file's name without its extension.
  pub name: String,
}
