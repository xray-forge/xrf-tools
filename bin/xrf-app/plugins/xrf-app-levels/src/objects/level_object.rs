use serde::Serialize;
use xrf_visual::VisualTransform;

use crate::objects::{LevelObjectId, LevelObjectKind, LevelObjectSpawn, LevelObjectVolume};

/// One discrete thing placed in a level: what it is, where it stands, and what its source records of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObject {
  pub id: LevelObjectId,
  pub kind: LevelObjectKind,
  pub name: String,
  /// Where it stands, its `XFORM` in renderer space; its shapes stand in the space it gives.
  pub transform: VisualTransform,
  /// What the spawn records of it, for an object the spawn places.
  pub spawn: Option<LevelObjectSpawn>,
  /// What it covers, for a logic volume.
  pub volume: Option<LevelObjectVolume>,
}
