use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::contract::world_ambient_effect_report::WorldAmbientEffectReport;

/// Where the weather's ambient effects near a viewport's camera stand.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAmbientReport {
  /// The effect playing, or none.
  pub effect: Option<WorldAmbientEffectReport>,
  /// Whether the camera stands indoors, where none starts.
  pub is_indoors: bool,
  /// Real time until the next may start, zero once it may.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub wait: Duration,
  /// How many have started since the level opened.
  pub played: u32,
}
