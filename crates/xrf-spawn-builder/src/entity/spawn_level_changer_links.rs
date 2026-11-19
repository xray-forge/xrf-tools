use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeBuiltDestination, AlifeBuiltFields, AlifeObject, AlifeObjectInherited, GraphVertex, SpawnGraphsChunk,
};

use crate::entity::spawn_level_entities::SpawnLevelEntities;
use crate::merge::spawn_merge_level::SpawnMergeLevel;

/// Level changer linking, `CLevelSpawnConstructor::fill_level_changers`: each changer takes the named graph point of
/// the level it leads to, the game vertex at that point and the vertex's node.
pub(crate) struct SpawnLevelChangerLinks;

impl SpawnLevelChangerLinks {
  /// Per-axis tolerance between a vertex's level point and its graph point, `Fvector::similar`.
  const SIMILAR: f32 = 0.001;

  /// Links every level changer of `entities`, beside its level in `levels`, through the merged `graph`.
  ///
  /// # Errors
  ///
  /// Returns an error when a level changer leads to a level not built, names a graph point its level lacks, or the
  /// point has no game vertex. xrAI's release build lists the first two unlinked and asserts on the third.
  pub(crate) fn fill(
    levels: &[SpawnMergeLevel<'_>],
    entities: &mut [SpawnLevelEntities],
    graph: &SpawnGraphsChunk,
  ) -> XrfResult {
    let targets: Vec<Vec<AlifeObject>> = entities.iter().map(|level| level.graph_points.clone()).collect();

    for (level, level_entities) in levels.iter().zip(entities.iter_mut()) {
      for object in &mut level_entities.objects {
        let AlifeObjectInherited::SeLevelChanger(changer) = &object.inherited else {
          continue;
        };
        let name: &str = &object.header.name;
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
        let mut fields: AlifeBuiltFields = object.get_built_fields();

        fields.destination = Some(AlifeBuiltDestination {
          game_vertex_id: vertex_id,
          level_vertex_id: vertex.level_vertex_id,
          position: point.header.position,
          direction: point.header.direction,
        });
        object.set_built_fields(&fields);
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
    // A game graph numbers its vertices in 16 bits, as the merge that built it checked.
    (0..=u16::MAX)
      .zip(&graph.vertices)
      .find(|(_, vertex)| vertex.level_id == level_id && vertex.level_point.similar(position, Self::SIMILAR))
  }
}
