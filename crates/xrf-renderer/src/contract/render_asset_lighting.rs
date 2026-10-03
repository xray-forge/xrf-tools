use serde::{Deserialize, Serialize};

/// How an asset viewer lights what it shows, in place of a level's weather: one light from a direction and a uniform
/// ambient, each a colour scaled by an intensity.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAssetLighting {
  /// Degrees above the horizon the light sits at, ninety directly overhead.
  pub sun_elevation: f32,
  /// Degrees around the vertical, zero looking along renderer `+z`.
  pub sun_azimuth: f32,
  pub sun_intensity: f32,
  /// The light's colour, each channel zero to one.
  pub sun_color: [f32; 3],
  pub ambient_intensity: f32,
  pub ambient_color: [f32; 3],
}
