use serde::Serialize;
use xrf_visual::VisualBounds;

use crate::scene::SceneDrawProblem;

/// What the page needs of a drawn scene beside its picture: the extent to frame a camera on, and each scene object it
/// does not draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneDrawDescription {
  /// The extent of the drawn objects' placed mesh boxes, in renderer space; none where the scene draws no object.
  pub bounds: Option<VisualBounds>,
  /// Each scene object not drawn, in scene object order.
  pub problems: Vec<SceneDrawProblem>,
}
