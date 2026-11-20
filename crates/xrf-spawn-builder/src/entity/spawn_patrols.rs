use xrf_level::{LevelGameFile, LevelGameWay, LevelGameWayPoint};
use xrf_math::Vector3d;
use xrf_spawn::{Patrol, PatrolLink, PatrolPoint};

use crate::entity::spawn_level_entities::SpawnLevelEntities;
use crate::entity::spawn_level_vertices::SpawnLevelVertices;
use crate::merge::spawn_merge_level::SpawnMergeLevel;

/// A level's patrol paths read from its `level.game`, `CPatrolPathStorage::load_raw`: points placed on the AI map,
/// links kept as they came.
pub(crate) struct SpawnPatrols;

impl SpawnPatrols {
  /// `u32(-1)`, the node of a point with none under it.
  const NO_NODE: u32 = u32::MAX;

  /// The level's ways as patrols, in the file's order.
  pub(crate) fn build(level: &SpawnMergeLevel<'_>, game: &LevelGameFile, vertices: &SpawnLevelVertices) -> Vec<Patrol> {
    game
      .ways
      .iter()
      .map(|way| Self::build_patrol(level, way, vertices))
      .collect()
  }

  fn build_patrol(level: &SpawnMergeLevel<'_>, way: &LevelGameWay, vertices: &SpawnLevelVertices) -> Patrol {
    let mut links: Vec<PatrolLink> = Vec::new();

    // `CGraphAbstract::save` writes each point's links in the order they were added, the points with none left out.
    for index in 0..way.points.len() as u32 {
      let targets: Vec<(u32, f32)> = way
        .links
        .iter()
        .filter(|link| u32::from(link.from) == index)
        .map(|link| (u32::from(link.to), link.probability))
        .collect();

      if !targets.is_empty() {
        links.push(PatrolLink { index, links: targets });
      }
    }

    Patrol {
      name: way.name.clone(),
      points: way
        .points
        .iter()
        .map(|point| Self::build_point(level, point, vertices))
        .collect(),
      links,
    }
  }

  /// `CPatrolPoint::load_raw` and `correct_position`: the node under the raised point and its game vertex, the point
  /// moved onto the node when outside its cell.
  ///
  /// A point with no node under it keeps none. xrAI leaves its game vertex uninitialised; this takes the nearest
  /// node's game vertex, one of its own level.
  fn build_point(level: &SpawnMergeLevel<'_>, point: &LevelGameWayPoint, vertices: &SpawnLevelVertices) -> PatrolPoint {
    let raised: Vector3d<f32> = SpawnLevelEntities::raise(&point.position);
    let node: Option<u32> = level
      .map
      .is_valid_position(&point.position)
      .then(|| level.map.find_node(&raised))
      .flatten();
    let game_vertex_id = |node: u32| vertices.first + level.cells[node as usize].game_vertex_id;

    match node {
      Some(node) => PatrolPoint {
        name: point.name.clone(),
        position: if level.map.is_inside(node, &point.position) {
          point.position
        } else {
          level.map.get_node_position(node)
        },
        flags: point.flags,
        level_vertex_id: node,
        game_vertex_id: game_vertex_id(node),
      },
      None => PatrolPoint {
        name: point.name.clone(),
        position: point.position,
        flags: point.flags,
        level_vertex_id: Self::NO_NODE,
        game_vertex_id: game_vertex_id(level.map.find_nearest_node(&raised)),
      },
    }
  }
}
