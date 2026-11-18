use serde::Serialize;
use xrf_visual::VisualBounds;

use crate::scene::{SceneHelperToolCount, SceneSpawnModelDescription};

/// What the page needs of a scene's helpers beside their picture: their extent, to frame a scene that draws only
/// helpers, each helped tool's count, and the spawn points drawn as models in place of an icon.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneHelpersDescription {
  /// The extent of every helper's shapes and icons and of each spawn model's position, in renderer space; none where
  /// the scene draws no helper.
  pub bounds: Option<VisualBounds>,
  /// Each helped tool helping at least one object, in the editor's tool order; spawn points drawn as models count.
  pub tools: Vec<SceneHelperToolCount>,
  /// Each spawn point drawn as its entity's model, in spawn point order.
  pub spawn_models: Vec<SceneSpawnModelDescription>,
}
