use serde::Serialize;

/// The object motion carrying a moving zone, which carries its idle light: `CTorridZone` moves along it, and
/// `UpdateIdleLight` stands the light `height` over where it has the zone.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightMotion {
  /// The motion, as the zone's spawn names it.
  pub name: String,
  /// The zone's `idle_light_height`.
  pub height: f32,
}
