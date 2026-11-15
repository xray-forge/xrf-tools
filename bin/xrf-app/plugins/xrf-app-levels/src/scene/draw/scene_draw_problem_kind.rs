use serde::Serialize;

/// Why a scene object is not drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SceneDrawProblemKind {
  /// Its reference names no `.object` in the scene's objects roots.
  Missing,
  /// Its `.object` was found and does not read.
  Unreadable,
}
