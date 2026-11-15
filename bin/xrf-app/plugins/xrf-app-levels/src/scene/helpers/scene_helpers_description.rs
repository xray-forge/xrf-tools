use serde::Serialize;
use xrf_visual::VisualBounds;

use crate::scene::{SceneHelperToolCount, SceneSpawnModelDescription};

/// What the page needs of a scene's helpers beside their picture: where they reach, to frame a camera on a scene that
/// draws them alone, how many each helped tool draws, and which spawn points stand as their models in place of an icon.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneHelpersDescription {
  /// The extent of every helper's shapes and icons and of where each spawn point drawn as a model stands, in renderer
  /// space; none where the scene draws no helper.
  pub bounds: Option<VisualBounds>,
  /// Each helped tool helping at least one object, in the editor's order of its tools, a spawn point drawn as its model
  /// counted.
  pub tools: Vec<SceneHelperToolCount>,
  /// Each spawn point drawn as its entity's model, in the scene's order of its spawn points.
  pub spawn_models: Vec<SceneSpawnModelDescription>,
}
