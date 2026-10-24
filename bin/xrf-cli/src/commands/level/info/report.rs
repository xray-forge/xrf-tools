use serde::Serialize;
use xrf_level::{LevelFile, LevelGeomFile, LevelVisualsChunk};

use crate::commands::level::info::level_geom_report::LevelGeomReport;
use crate::commands::level::info::level_header_report::LevelHeaderReport;
use crate::commands::level::info::level_shaders_report::LevelShadersReport;
use crate::commands::level::info::level_structure_report::LevelStructureReport;
use crate::commands::level::info::level_visuals_report::LevelVisualsReport;

/// What `level info` read out of a compiled level.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfoReport {
  header: LevelHeaderReport,
  shaders: LevelShadersReport,
  /// Absent for a level carrying no visuals chunk, which is a level that draws nothing.
  visuals: Option<LevelVisualsReport>,
  /// `level.geom`, absent when the directory holds none.
  geometry: Option<LevelGeomReport>,
  /// `level.geomx`, the fast-path twin, absent when the directory holds none.
  detail_geometry: Option<LevelGeomReport>,
  structure: LevelStructureReport,
}

impl LevelInfoReport {
  pub fn new(
    level: &LevelFile,
    visuals: Option<&LevelVisualsChunk>,
    geometry: Option<&LevelGeomFile>,
    detail_geometry: Option<&LevelGeomFile>,
  ) -> Self {
    Self {
      header: LevelHeaderReport::of(level),
      shaders: LevelShadersReport::of(level),
      visuals: visuals.map(LevelVisualsReport::of),
      geometry: geometry.map(LevelGeomReport::of),
      detail_geometry: detail_geometry.map(LevelGeomReport::of),
      structure: LevelStructureReport::of(level),
    }
  }
}
