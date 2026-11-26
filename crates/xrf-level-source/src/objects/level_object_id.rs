use serde::{Deserialize, Serialize};

use crate::objects::LevelPlacementFile;

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
  /// A record of one of the level's companion files, by its index among the file's records.
  Placement { file: LevelPlacementFile, index: u32 },
  /// One of the level's static lights, by its index among the bundle's.
  Light { index: u32 },
}
