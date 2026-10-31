use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_spawn::SpawnLevelGraph;
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::objects::LevelObjectsDescription;
use crate::parts::LevelGraphPart;
use crate::viewer::map::annotations::{AnnotationMarker, AnnotationPatrol, AnnotationVolume};

/// Everything a level map's annotations can draw of a level's objects, engine space seen from above: what covers a
/// space by its outlines, the patrol paths by their links, and what stands at a point by a dot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelAnnotations {
  pub volumes: Vec<AnnotationVolume>,
  pub patrols: Vec<AnnotationPatrol>,
  pub markers: Vec<AnnotationMarker>,
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
      markers: objects.objects.iter().filter_map(AnnotationMarker::of).collect(),
    }
  }
}
