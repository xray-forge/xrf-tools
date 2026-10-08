use serde::{Deserialize, Serialize};

/// The screen-space indirect light as gathered.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedIndirectLight {
  /// How much of the bounced light is added.
  pub intensity: f32,
  /// Whether VBAO's own search gathers it, rather than a search of its own.
  pub is_shared: bool,
}
