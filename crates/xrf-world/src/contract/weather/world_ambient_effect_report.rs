use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The weather's ambient effect playing near a viewport's camera.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAmbientEffectReport {
  /// Its `effects.ltx` section.
  pub name: String,
  /// The `particles.xr` effect or group it plays.
  pub particles: String,
  /// Real time its life has left, zero while what it emitted dies out.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub remaining: Duration,
}
