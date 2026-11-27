use std::collections::BTreeMap;

use xrf_math::glam::Vec3;
use xrf_spawn::SpawnPatrolsChunk;

use crate::id::game_vertex_id::GameVertexId;
use crate::patrol::game_patrol_point::GamePatrolPoint;

/// The patrol paths the spawn lays over every level, by name (`CPatrolPathStorage`): the points scripts send objects
/// to, such as where a story squad spawns (`patrol(spawn_point):point(0)`). Their links are not kept: nothing offline
/// walks a patrol yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GamePatrols {
  patrols: BTreeMap<String, Vec<GamePatrolPoint>>,
}

impl GamePatrols {
  /// The patrols the spawn's patrols chunk holds.
  pub fn of_spawn(chunk: &SpawnPatrolsChunk) -> Self {
    Self {
      patrols: chunk
        .patrols
        .iter()
        .map(|patrol| {
          let points: Vec<GamePatrolPoint> = patrol
            .points
            .iter()
            .map(|point| GamePatrolPoint {
              name: point.name.clone(),
              position: Vec3::new(point.position.x, point.position.y, point.position.z),
              level_vertex: point.level_vertex_id,
              game_vertex: GameVertexId(point.game_vertex_id),
            })
            .collect();

          (patrol.name.clone(), points)
        })
        .collect(),
    }
  }

  /// How many patrols it holds.
  pub fn len(&self) -> usize {
    self.patrols.len()
  }

  pub fn is_empty(&self) -> bool {
    self.patrols.is_empty()
  }

  /// The points of the patrol `name`, in order; none for a patrol the spawn lacks.
  pub fn get(&self, name: &str) -> Option<&[GamePatrolPoint]> {
    self.patrols.get(name).map(Vec::as_slice)
  }
}
