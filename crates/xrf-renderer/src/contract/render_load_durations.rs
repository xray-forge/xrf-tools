use std::time::Duration;

use serde::{Deserialize, Serialize};

/// How long a viewport's level had been opening when each part of it finished, each noted the first frame it is seen
/// finished; none for a part not finished yet.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLoadDurations {
  /// Every sector taken in or failed.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub sectors: Option<Duration>,
  /// Every spawned object's model in the scene.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub spawn: Option<Duration>,
  /// The grass read.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub grass: Option<Duration>,
  /// The local lights read.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub lights: Option<Duration>,
  /// The particle systems read.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub particles: Option<Duration>,
  /// Everything resident, every texture settled last: the whole load.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub ready: Option<Duration>,
}
