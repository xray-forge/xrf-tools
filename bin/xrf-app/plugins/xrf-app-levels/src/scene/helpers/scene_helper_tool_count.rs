use serde::Serialize;

use crate::scene::SceneToolKind;

/// How many of one helped tool's objects draw a helper in a scene.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneHelperToolCount {
  pub tool: SceneToolKind,
  pub count: u32,
}
