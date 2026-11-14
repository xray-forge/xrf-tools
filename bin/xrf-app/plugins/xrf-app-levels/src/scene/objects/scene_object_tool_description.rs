use serde::Serialize;

use crate::scene::{SceneObjectDescription, SceneToolKind};

/// One object tool of a scene and its objects, in the order the editor loads them: the groups' members of its class,
/// then its own part's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectToolDescription {
  pub tool: SceneToolKind,
  pub objects: Vec<SceneObjectDescription>,
}
