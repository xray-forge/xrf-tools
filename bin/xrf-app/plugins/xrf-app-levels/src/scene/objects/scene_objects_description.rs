use serde::Serialize;

use crate::scene::{SceneObjectToolDescription, SceneProblem};

/// What a scene's object tools hold, typed: every tool with a part or a group's member, and why any tool's objects did
/// not read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectsDescription {
  /// The tools whose objects read, by name.
  pub tools: Vec<SceneObjectToolDescription>,
  /// The tools whose objects did not read, each left out of [`Self::tools`].
  pub problems: Vec<SceneProblem>,
}
