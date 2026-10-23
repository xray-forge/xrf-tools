use std::sync::Arc;

use xrf_chunk::InMemoryChunkDataSource;
use xrf_engine_target::XrayEngineResolution;
use xrf_level::{LevelFile, LevelGeomSource, LevelSector, LevelVisualsChunk};
use xrf_ltx::LtxDialect;
use xrf_material::XraySurfaceDescriptor;
use xrf_vfs::{XrayRoots, XraySearch};
use xrf_visual::SectorOutline;

use crate::core::session::SessionSnapshot;
use crate::plugins::levels::start::get_level_start;
use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::selection::level_map_description::LevelMapDescription;
use crate::plugins::levels::state::selection::level_start::LevelStart;
use crate::plugins::levels::state::selection::level_sun_description::LevelSunDescription;
use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::selected_level_description::SelectedLevelDescription;

/// The level a viewer has open as the open read it, which a later call reads without opening it again.
///
/// What is read from it the first time anything asks, such as its spawn, collision form or environment, is kept as
/// its session snapshot's parts by the module that reads it, rather than here.
pub struct SelectedLevel {
  pub source: LevelSource,
  /// What each texture reference the level's surfaces bind came to, decided at open.
  pub textures: Vec<LevelTextureReference>,
  /// How the renderer draws each entry of the shader table, in its order, decided at open.
  pub surfaces: Vec<XraySurfaceDescriptor>,
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
  /// Where its map lies over it, read at open; `None` where its config places none.
  pub level_map: Option<LevelMapDescription>,
  /// Render geometry with its payloads still on the heap where they were read, serving whichever range a sector
  /// names. Shared rather than locked: a range is read without moving the source, so sectors pack side by side.
  pub geometry: LevelGeomSource<InMemoryChunkDataSource>,
}

impl SelectedLevel {
  /// What the viewer is shown of the open level under its snapshot's identity, where it starts decided again through
  /// `search` where a commit let the start go.
  pub fn describe_at(
    current: &SessionSnapshot<Self>,
    search: &XraySearch,
  ) -> SessionSnapshot<SelectedLevelDescription> {
    current.answer(current.describe(get_level_start(current, search)))
  }

  /// What the viewer is shown of the open level, which opens at `start`, its start part.
  pub fn describe(&self, start: Option<LevelStart>) -> SelectedLevelDescription {
    let level: &LevelFile = &self.level;

    SelectedLevelDescription {
      bounds: SectorOutline::merge_bounds(&self.outlines),
      drawables: self.visuals.count_drawable() as u32,
      engine: self.engine.clone(),
      has_sun: level.lights.as_ref().is_some_and(|it| it.get_sun().is_some()),
      sun: LevelSunDescription::of(level.lights.as_ref().and_then(|it| it.get_sun())),
      level_map: self.level_map.clone(),
      lights: level.lights.as_ref().map_or(0, |it| it.lights.len()) as u32,
      portals: level.portals.as_ref().map_or(0, |it| it.portals.len()) as u32,
      roots: self.roots.clone(),
      sectors: self.outlines.clone(),
      start,
      shader_entries: level.shaders.as_ref().map_or(0, |it| it.entries.len()) as u32,
      source: self.source.clone(),
      surfaces: self.surfaces.clone(),
      textures: self.textures.clone(),
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
