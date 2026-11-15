use serde::Serialize;
use xrf_visual::VisualBounds;

use crate::scene::SceneDrawProblem;

/// What the page needs of a drawn scene beside its picture: where its drawn objects stand, to frame a camera on, and
/// each scene object it does not draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneDrawDescription {
  /// The extent of the scene objects drawn, each `.object`'s meshes' boxes placed, in renderer space; none where the
  /// scene draws no object.
  pub bounds: Option<VisualBounds>,
  /// Each scene object not drawn, in the scene's order of its objects.
  pub problems: Vec<SceneDrawProblem>,
}
