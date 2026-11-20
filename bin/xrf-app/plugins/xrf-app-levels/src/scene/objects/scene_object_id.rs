use serde::{Deserialize, Serialize};

use crate::scene::SceneToolKind;

/// Names one scene object until its scene is read again: its tool, and its index in the tool's objects in editor load
/// order, group members first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectId {
  pub tool: SceneToolKind,
  pub index: u32,
}
