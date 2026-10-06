use serde::{Deserialize, Serialize};

/// How trees and grass move in the wind.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderFoliageMode {
  /// The engine's own: each tree leant by a wind turning through the weather's `trees_*` keys, each waving tuft by
  /// its wave.
  #[default]
  Engine,
  /// Trunks swinging downwind and branches and grass carried by a flow field drifting with the weather's wind.
  Enhanced,
}
