use serde::Serialize;

use crate::scene::{SceneObjectToolDescription, SceneProblem};

/// A scene's object tools' typed objects: every tool with a part or a group member, and why any tool's objects did
/// not read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectsDescription {
  /// The tools whose objects read, in the editor's tool order.
  pub tools: Vec<SceneObjectToolDescription>,
  /// The tools whose objects did not read, each left out of [`Self::tools`].
  pub problems: Vec<SceneProblem>,
}
