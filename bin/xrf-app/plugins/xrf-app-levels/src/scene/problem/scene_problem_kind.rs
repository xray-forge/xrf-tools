use serde::Serialize;

/// Why the Level Editor would skip or refuse one of a scene's parts, as it does rather than refusing the scene.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SceneProblemKind {
  /// The file does not read as a part, or what its tool keeps in it does not read as that tool's.
  Unreadable,
  /// The part carries another scene's GUID: a stale copy of the tool's part, which the editor skips.
  ForeignGuid,
  /// The part's `[main]` holds a tool version other than the one its tool reads.
  WrongVersion,
  /// The part is saved as another tool's class, or holds an object of another class than its tool's.
  ForeignClass,
}
