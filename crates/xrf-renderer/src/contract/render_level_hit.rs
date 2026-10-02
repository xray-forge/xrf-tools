use serde::{Deserialize, Serialize};

/// What of a level is drawn under a point of a viewport, and where the ray from the eye met it, in renderer space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderLevelHit {
  /// A surface the level compiled.
  Surface {
    sector: u32,
    /// The shader table entry drawing it.
    shader_id: u32,
    /// The sector's instanced mesh it is one place of, or none for its baked geometry.
    mesh: Option<u32>,
    /// Which place of the mesh it is, or none for the baked geometry.
    place: Option<u32>,
    /// Whether it is a clump of trees drawn as its impostor.
    is_impostor: bool,
    point: [f32; 3],
  },
  /// An object the level's spawn places.
  Spawn {
    /// Its index among the level's spawned objects.
    object: u32,
    point: [f32; 3],
  },
}
