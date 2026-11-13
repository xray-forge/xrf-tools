use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_spawn::{AlifeObject, AlifeObjectInherited, GraphVertex, SpawnGraphsChunk};

use crate::spawn::spawn_level_entities::SpawnLevelEntities;
use crate::spawn::spawn_merge_level::SpawnMergeLevel;

/// `fill_level_changers` (`level_spawn_constructor.cpp:536`): every level changer takes, from the level it leads to,
/// the graph point it names, the game vertex standing at that point and the vertex's node.
pub(crate) struct SpawnLevelChangerLinks;

impl SpawnLevelChangerLinks {
  /// How near, on each axis, a vertex's level point stands to the graph point it was made from: within it, not at it,
  /// `Fvector::similar`.
  const SIMILAR: f32 = 0.001;

  /// Links every level changer of `entities`, beside its level in `levels`, through the merged `graph`.
  ///
  /// # Errors
  ///
  /// Returns an error when a level changer leads to a level not built, names a graph point its level lacks, or the
  /// point has no game vertex: xrAI's release build leaves such a changer unlinked and lists it.
  pub(crate) fn fill(
    levels: &[SpawnMergeLevel<'_>],
    entities: &mut [SpawnLevelEntities],
    graph: &SpawnGraphsChunk,
  ) -> XrfResult {
    let targets: Vec<Vec<AlifeObject>> = entities.iter().map(|level| level.graph_points.clone()).collect();

    for (level, level_entities) in levels.iter().zip(entities.iter_mut()) {
      for object in &mut level_entities.objects {
        let name: String = object.header.name.clone();
        let AlifeObjectInherited::SeLevelChanger(changer) = &mut object.inherited else {
          continue;
        };
        let Some(target) = levels.iter().position(|level| level.name == changer.dest_level_name) else {
          return Err(XrfError::new_invalid_error(format!(
            "Level changer {name} on level {} leads to level {}, which is not built",
            level.name, changer.dest_level_name
          )));
        };
        let Some(point) = targets[target]
          .iter()
          .find(|point| point.header.name == changer.dest_graph_point)
        else {
          return Err(XrfError::new_invalid_error(format!(
            "Level changer {name} on level {} names graph point {}, which level {} lacks",
            level.name, changer.dest_graph_point, changer.dest_level_name
          )));
        };
        let Some((vertex_id, vertex)) = Self::find_vertex(graph, levels[target].id, &point.header.position) else {
          return Err(XrfError::new_invalid_error(format!(
            "Level changer {name} on level {} leads to graph point {} of level {}, which has no game vertex",
            level.name, changer.dest_graph_point, changer.dest_level_name
          )));
        };

        changer.dest_game_vertex_id = vertex_id;
        changer.dest_position = point.header.position;
        changer.dest_direction = point.header.direction;
        changer.dest_level_vertex_id = vertex.level_vertex_id;
      }
    }

    Ok(())
  }

  /// The first vertex of the level whose level point stands at `position`.
  fn find_vertex<'a>(
    graph: &'a SpawnGraphsChunk,
    level_id: u8,
    position: &Vector3d<f32>,
  ) -> Option<(u16, &'a GraphVertex)> {
    let similar = |a: f32, b: f32| (a - b).abs() < Self::SIMILAR;

    graph
      .vertices
      .iter()
      .enumerate()
      .find(|(_, vertex)| {
        vertex.level_id == level_id
          && similar(vertex.level_point.x, position.x)
          && similar(vertex.level_point.y, position.y)
          && similar(vertex.level_point.z, position.z)
      })
      .map(|(index, vertex)| (index as u16, vertex))
  }
}
