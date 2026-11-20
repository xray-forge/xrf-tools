use serde::{Deserialize, Serialize};

use crate::level::geometry::level_geometry_census::LevelGeometryCensus;
use crate::provenance::level_scene_instances::LevelSceneInstances;
use crate::provenance::level_scene_surfaces::LevelSceneSurfaces;

/// The objects a scene's static geometry came back as, and what the decompile counted of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneGeometry {
  /// Each object written, by its path under `$objects$` without the extension.
  pub objects: Vec<String>,
  pub census: LevelGeometryCensus,
  /// How the surfaces were dressed, where they were inferred.
  #[serde(default)]
  pub surfaces: Option<LevelSceneSurfaces>,
  /// The library objects found placed in the static geometry, where libraries were looked in.
  #[serde(default)]
  pub instances: Option<LevelSceneInstances>,
}
