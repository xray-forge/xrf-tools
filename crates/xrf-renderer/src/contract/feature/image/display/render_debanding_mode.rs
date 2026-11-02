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
  /// Sky debanding: the sky's cubes and clouds smoothed where their stored steps band, each texel averaged with
  /// neighbours less than a step apart, before the sky is drawn with them.
  Enhanced,
}
