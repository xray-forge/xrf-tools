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
  /// Real seconds of its life left; none while what it emitted dies out.
  pub remaining: f32,
}
