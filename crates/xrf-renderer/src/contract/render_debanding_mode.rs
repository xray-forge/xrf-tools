use serde::{Deserialize, Serialize};

/// Whether the sky's gradients are smoothed where its colours fall into visible bands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDebandingMode {
  /// The engine's own: the sky as drawn.
  #[default]
  Engine,
  /// Sky debanding: each sky pixel averaged with neighbours scattered around it wherever they differ from it by less
  /// than a band's step.
  Enhanced,
}
