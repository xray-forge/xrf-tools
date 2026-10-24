use xrf_spawn::SpawnLevelGraph;

use crate::parts::LevelSpawn;
use crate::viewer::map::annotations::{AnnotationOutline, AnnotationPatrol, AnnotationPoint, AnnotationVolume};

/// Everything a level map's annotations can draw of a level, engine space seen from above.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelAnnotations {
  pub volumes: Vec<AnnotationVolume>,
  pub patrols: Vec<AnnotationPatrol>,
  /// The game graph's vertices on the level, by id.
  pub graph_points: Vec<(u16, AnnotationPoint)>,
}

impl LevelAnnotations {
  /// What `spawn` and `graph` hold of the level named `level`.
  pub fn of(spawn: &LevelSpawn, graph: &SpawnLevelGraph, level: &str) -> Self {
    Self {
      volumes: spawn
        .objects
        .iter()
        .filter_map(|object| AnnotationVolume::of(object, graph, level))
        .collect(),
      patrols: graph.patrols.iter().map(AnnotationPatrol::of).collect(),
      graph_points: graph
        .points
        .iter()
        .map(|(id, point)| (*id, AnnotationOutline::to_point(point)))
        .collect(),
    }
  }
}
