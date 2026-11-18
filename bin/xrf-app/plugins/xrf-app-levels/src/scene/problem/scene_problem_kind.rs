use serde::Serialize;

/// Why the Level Editor would skip or refuse one of a scene's parts while still opening the scene. A part carrying
/// another scene's GUID is none of these: it refuses the scene.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SceneProblemKind {
  /// The file does not read as a part, or its content does not read as its tool's.
  Unreadable,
  /// The part's `[main]` holds a tool version other than the one its tool reads.
  WrongVersion,
  /// The part is saved as another tool's class, or holds an object of another class than its tool's.
  ForeignClass,
}
