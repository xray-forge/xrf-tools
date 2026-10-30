use serde::{Deserialize, Serialize};

/// What names one level object for as long as its level is open, by where it is read from: the selection, a pick, the
/// Outliner's rows and the renderer's helpers all address objects by it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "source", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelObjectId {
  /// An object of the level's spawn, by its index among them.
  Spawn { index: u32 },
  /// A patrol path starting on the level, by its index among them in the spawn's order.
  Patrol { path: u32 },
  /// One point of a patrol path, by its index along the path.
  PatrolPoint { path: u32, point: u32 },
  /// A game graph vertex on the level, by its id.
  GraphPoint { vertex: u16 },
}
