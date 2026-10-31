use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_spawn::SpawnLevelGraph;
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::objects::{LevelObject, LevelObjectId, LevelObjectsDescription};
use crate::parts::LevelGraphPart;
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
  /// What the open level's objects hold of it, its graph placing the levels a changer leads to, read through `search`
  /// where they are not yet.
  ///
  /// # Errors
  ///
  /// Returns why the level has no name to find its share by, or why its spawn or graph cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    let graph: Arc<SpawnLevelGraph> = LevelGraphPart::get(current, search)?;

    Ok(Self::of(
      &LevelObjectsDescription::read(current, search)?,
      &graph,
      &current.source.require_name()?,
    ))
  }

  /// What `objects` hold of the level named `level`, `graph` placing the levels a changer leads to.
  pub fn of(objects: &LevelObjectsDescription, graph: &SpawnLevelGraph, level: &str) -> Self {
    Self {
      volumes: objects
        .objects
        .iter()
        .filter_map(|object| AnnotationVolume::of(object, graph, level))
        .collect(),
      patrols: objects.objects.iter().filter_map(AnnotationPatrol::of).collect(),
      graph_points: objects
        .objects
        .iter()
        .filter_map(|object: &LevelObject| match object.id {
          LevelObjectId::GraphPoint { vertex } => {
            Some((vertex, AnnotationOutline::from_renderer_point(&object.transform.c)))
          }
          _ => None,
        })
        .collect(),
    }
  }
}
