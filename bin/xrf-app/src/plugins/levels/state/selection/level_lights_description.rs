use serde::Serialize;
use xrf_visual::LightsDescription;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// A level's lights, and what each projector its spots name resolved to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelLightsDescription {
  pub lights: LightsDescription,
  /// By the lights' projector index.
  pub projectors: Vec<LevelTextureReference>,
}
