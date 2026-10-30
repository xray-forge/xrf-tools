use serde::Serialize;
use xrf_spawn::PatrolPoint;

/// What the spawn records of one patrol point beyond its name and place: its flags, which a logic scheme reads, and
/// the level and game graph vertices it stands at.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectPatrolPoint {
  pub flags: u32,
  pub level_vertex: u32,
  pub game_vertex: u16,
}

impl From<&PatrolPoint> for LevelObjectPatrolPoint {
  fn from(point: &PatrolPoint) -> Self {
    Self {
      flags: point.flags,
      level_vertex: point.level_vertex_id,
      game_vertex: point.game_vertex_id,
    }
  }
}
