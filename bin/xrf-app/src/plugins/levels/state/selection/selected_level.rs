use std::sync::Arc;

use xrf_engine_target::XrayEngineResolution;
use xrf_level::{LevelFile, LevelSector, LevelVisualsChunk};
use xrf_ltx::LtxDialect;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::selection::level_sun_description::LevelSunDescription;
use crate::plugins::levels::state::selection::selected_level_description::SelectedLevelDescription;

/// The level a viewer has open as the open read it: what names it, and the `level` bundle every part reads, which a
/// later call reads without opening it again.
///
/// What is read from other files, such as its map, geometry, surfaces, spawn, collision form or environment, is kept as
/// its session snapshot's parts by the part type that reads it (`plugins/levels/parts/`), rather than here, so a change
/// to one of them moves that part and not the level.
pub struct SelectedLevel {
  pub source: LevelSource,
  /// The roots the level was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  /// The rules the game's configs are resolved with, as the open was asked to read them.
  pub dialect: Arc<dyn LtxDialect>,
  /// The engine the game's configs are read as, where the engines read them differently, and what decided it.
  pub engine: XrayEngineResolution,
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  /// What each sector is and where, taken at open from what the visuals declare, so a viewer can decide what to
  /// stream before reading any geometry.
  pub outlines: Vec<SectorOutline>,
}

impl SelectedLevel {
  /// What the viewer is shown of the open level itself, its root, which reads nothing: each part it shows beside it has
  /// a query of its own.
  pub fn describe(&self) -> SelectedLevelDescription {
    let level: &LevelFile = &self.level;

    SelectedLevelDescription {
      bounds: SectorOutline::merge_bounds(&self.outlines),
      drawables: self.visuals.count_drawable() as u32,
      engine: self.engine.clone(),
      has_sun: level.lights.as_ref().is_some_and(|it| it.get_sun().is_some()),
      sun: LevelSunDescription::of(level.lights.as_ref().and_then(|it| it.get_sun())),
      lights: level.lights.as_ref().map_or(0, |it| it.lights.len()) as u32,
      portals: level.portals.as_ref().map_or(0, |it| it.portals.len()) as u32,
      roots: self.roots.clone(),
      sectors: self.outlines.clone(),
      shader_entries: level.shaders.as_ref().map_or(0, |it| it.entries.len()) as u32,
      source: self.source.clone(),
      visuals: self.visuals.visuals.len() as u32,
      xrlc_quality: level.header.xrlc_quality,
      xrlc_version: level.header.xrlc_version,
    }
  }

  /// The sectors the level names, which is what the outlines were taken from.
  pub fn get_sectors(&self) -> &[LevelSector] {
    self.level.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors)
  }
}
