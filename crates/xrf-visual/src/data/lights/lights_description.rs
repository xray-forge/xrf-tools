use serde::Serialize;

use crate::data::lights::light_animator_description::LightAnimatorDescription;
use crate::data::lights::light_description::LightDescription;

/// A level's lights: its spawned lamps and its own, with the animations and projectors they name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightsDescription {
  pub lights: Vec<LightDescription>,
  pub animators: Vec<LightAnimatorDescription>,
  /// The projector textures the spots name, as the lamps reference them.
  pub projectors: Vec<String>,
}
