use serde::{Deserialize, Serialize};

use crate::level::surfaces::level_surface_census::LevelSurfaceCensus;

/// The surfaces of a scene's static geometry as inferred, from the libraries its geometry records, and every compiler
/// shader chosen with why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSurfaces {
  /// The object library roots whose pairings broke ties, as given; a file there that could not be used is among the
  /// scene's refusals.
  pub object_roots: Vec<String>,
  pub census: LevelSurfaceCensus,
}
