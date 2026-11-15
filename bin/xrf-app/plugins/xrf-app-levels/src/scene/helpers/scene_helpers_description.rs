use serde::Serialize;
use xrf_visual::VisualBounds;

use crate::scene::SceneHelperToolCount;

/// What the page needs of a scene's helpers beside their picture: where they reach, to frame a camera on a scene that
/// draws them alone, and how many each helped tool draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneHelpersDescription {
  /// The extent of every helper's shapes and icons, in renderer space; none where the scene draws no helper.
  pub bounds: Option<VisualBounds>,
  /// Each helped tool drawing at least one helper, in the editor's order of its tools.
  pub tools: Vec<SceneHelperToolCount>,
}
