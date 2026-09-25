use serde::Serialize;

use crate::data::lights::light_animator_key::LightAnimatorKey;

/// A colour animation of `lanims.xr` (`CLAItem`), which replaces the colour of every light it drives.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimatorDescription {
  pub name: String,
  pub fps: f32,
  pub frame_count: u32,
  /// By frame, the first at frame zero.
  pub keys: Vec<LightAnimatorKey>,
}
