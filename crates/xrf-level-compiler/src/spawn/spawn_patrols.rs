use xrf_level::{LevelGameFile, LevelGameWay, LevelGameWayPoint};
use xrf_math::Vector3d;
use xrf_spawn::{Patrol, PatrolLink, PatrolPoint};

use crate::spawn::spawn_level_entities::{SpawnLevelEntities, SpawnLevelVertices};
use crate::spawn::spawn_merge_level::SpawnMergeLevel;

/// A level's patrol paths as xrAI reads them from its `level.game` (`CPatrolPathStorage::load_raw`,
/// `patrol_path_storage.cpp:20`): each way's points placed on the AI map, its links kept as they came.
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

  /// `CPatrolPoint::load_raw` and `correct_position`: the node of the point's cell under its raised position, the
  /// point moved onto that node where it stands outside its cell, and the node's game vertex.
  ///
  /// A point with no node under it keeps none; xrAI leaves its game vertex as memory held it, where this takes the
  /// game vertex of its nearest node's cell, a vertex of its own level.
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
