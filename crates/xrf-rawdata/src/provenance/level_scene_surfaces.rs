use serde::{Deserialize, Serialize};

use crate::level::surfaces::level_surface_census::LevelSurfaceCensus;
use crate::provenance::level_scene_surface_libraries::LevelSceneSurfaceLibraries;

/// The surfaces of a scene's static geometry as inferred: from what, and every compiler shader chosen with
/// why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSurfaces {
  pub libraries: LevelSceneSurfaceLibraries,
  pub census: LevelSurfaceCensus,
}
