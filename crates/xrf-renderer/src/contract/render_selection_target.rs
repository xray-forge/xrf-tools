use serde::{Deserialize, Serialize};

/// One thing of a level a selection names, as a pick names it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderSelectionTarget {
  /// An object the level's spawn places, by its index among them.
  Spawn { object: u32 },
  /// A surface the level compiled: one place of a sector's instanced mesh, or the sector's baked geometry of one shader
  /// table entry.
  Surface {
    sector: u32,
    shader_id: u32,
    mesh: Option<u32>,
    place: Option<u32>,
  },
}
