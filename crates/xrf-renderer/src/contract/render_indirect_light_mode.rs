use serde::{Deserialize, Serialize};

/// Whether the light surfaces bounce onto each other is gathered from what the frame shows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderIndirectLightMode {
  /// The engine's own: the hemisphere and the ambient alone stand for every bounce.
  #[default]
  Engine,
  /// Screen-space indirect light: the sunlight and lamplight on what the frame shows, bounced once onto what faces it.
  Enhanced,
}
