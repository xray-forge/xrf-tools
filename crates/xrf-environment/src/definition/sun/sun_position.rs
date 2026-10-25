use serde::Serialize;

/// Where the sun stands at one hour of Monolith's sun table, in degrees, named as the table names them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SunPosition {
  /// `sun_altitude`, which `setHP` makes the heading.
  pub altitude: f32,
  /// `sun_longitude`, which `setHP` makes the pitch.
  pub longitude: f32,
}
