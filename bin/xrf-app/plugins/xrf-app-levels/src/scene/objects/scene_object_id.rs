use serde::{Deserialize, Serialize};

use crate::scene::SceneToolKind;

/// What names one scene object for as long as its scene's read stands: its tool, and its place in the tool's objects as
/// the editor loads them, the groups' members first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectId {
  pub tool: SceneToolKind,
  pub index: u32,
}
