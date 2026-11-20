use serde::Serialize;

use crate::level::geometry::level_geometry_census::LevelGeometryCensus;
use crate::level::scene::level_decompile_surfaces::LevelDecompileSurfaces;

/// The static geometry a decompile wrote, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileGeometry {
  /// The objects written, one per sector drawing static geometry.
  pub objects: usize,
  #[serde(flatten)]
  pub census: LevelGeometryCensus,
  /// How the surfaces were dressed, where they were inferred; the sidecar lists each compiler shader chosen.
  pub surfaces: Option<LevelDecompileSurfaces>,
}
