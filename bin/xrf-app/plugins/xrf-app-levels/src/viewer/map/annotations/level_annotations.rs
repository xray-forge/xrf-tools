use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_spawn::SpawnLevelGraph;
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::objects::LevelObjectsDescription;
use crate::parts::{LevelGraphPart, LevelSpawnPart};
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
  /// What the open level's objects and graph part hold of it, read through `search` where they are not yet.
  ///
  /// # Errors
  ///
  /// Returns why the level has no name to find its share by, or why its spawn or graph cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    Ok(Self::of(
      &LevelObjectsDescription::describe(&*LevelSpawnPart::get(current, search)?),
      &*LevelGraphPart::get(current, search)?,
      &current.source.require_name()?,
    ))
  }

  /// What `objects` and `graph` hold of the level named `level`.
  pub fn of(objects: &LevelObjectsDescription, graph: &SpawnLevelGraph, level: &str) -> Self {
    Self {
      volumes: objects
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
