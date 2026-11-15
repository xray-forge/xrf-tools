use serde::Serialize;

use crate::scene::{SceneDrawProblemKind, SceneObjectId};

/// One scene object the scene does not draw, and why.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneDrawProblem {
  /// The scene object, as the scene's objects name it.
  pub object: SceneObjectId,
  /// The library object it places, a path under the objects roots without its extension.
  pub reference: String,
  pub kind: SceneDrawProblemKind,
  pub message: String,
}
