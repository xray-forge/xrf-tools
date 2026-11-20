use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// Whether the sky's gradients are smoothed where its colours fall into visible bands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDebandingMode {
  /// The engine's own: the sky as drawn.
  #[default]
  Engine,
  /// Smooths banding in the sky's cubes and clouds by averaging each texel with neighbours less than a step apart.
  Enhanced,
}
