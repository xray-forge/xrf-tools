use serde::Serialize;

/// What one binary tool of a scene holds, counted: its payload stays in the backend. Told apart by its tool, spelled as
/// that tool's `SceneToolKind`: the name its part files carry.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "tool", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub(crate) enum SceneBinaryToolDescription {
  /// `ai_map.part`: the nodes the AI walks.
  AiMap { nodes: u32 },
  /// `detail_object.part`: the detail objects the library plants, and the grid's slots planting them.
  DetailObject { objects: u32, slots: u32 },
  /// `wallmark.part`: the decals, in slots of one shader and texture each.
  Wallmark { slots: u32, marks: u32 },
}
