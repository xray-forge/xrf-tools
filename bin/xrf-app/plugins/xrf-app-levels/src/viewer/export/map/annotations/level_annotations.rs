use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_level_source::{LevelGraphPart, LevelObjectsDescription, SelectedLevel};
use xrf_spawn::SpawnLevelGraph;
use xrf_vfs::XraySearch;

use crate::viewer::export::map::annotations::{AnnotationMarker, AnnotationPatrol, AnnotationVolume};

/// A level's objects as map annotations, in engine space from above: volumes by outline, patrol paths by their links,
/// and point objects by a dot.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LevelAnnotations {
  pub volumes: Vec<AnnotationVolume>,
  pub patrols: Vec<AnnotationPatrol>,
  pub markers: Vec<AnnotationMarker>,
}

impl LevelAnnotations {
  /// The open level's annotations, with its graph placing the levels its changers lead to.
  ///
  /// # Errors
  ///
  /// Returns an error when the level has no name, or its spawn or graph cannot be read.
  pub(crate) fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    let graph: Arc<SpawnLevelGraph> = LevelGraphPart::get(current, search)?;

    Ok(Self::of(
      &LevelObjectsDescription::read(current, search)?,
      &graph,
      &current.source.require_name()?,
    ))
  }

  /// The annotations of `objects` on the level named `level`, with `graph` placing the levels its changers lead to.
  pub(crate) fn of(objects: &LevelObjectsDescription, graph: &SpawnLevelGraph, level: &str) -> Self {
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
