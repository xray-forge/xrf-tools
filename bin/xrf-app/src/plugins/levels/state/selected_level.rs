use std::sync::Mutex;

use xrf_chunk::InMemoryChunkDataSource;
use xrf_level::{LevelFile, LevelGeomSource, LevelSector, LevelVisualsChunk};
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::core::session::Session;
use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::packed_sector::PackedSector;
use crate::plugins::levels::state::selected_level_description::SelectedLevelDescription;

/// The level a viewer has open, and everything a later call reads without opening it again.
pub struct SelectedLevel {
  pub source: LevelSource,
  /// What each texture reference of the shader table came to, decided at open.
  pub textures: Vec<LevelTextureReference>,
  /// The roots the level was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  /// What each sector is and where, taken at open from what the visuals declare, so a viewer can decide what to
  /// stream before reading any geometry.
  pub outlines: Vec<SectorOutline>,
  /// Render geometry with its payloads still on the heap where they were read, serving whichever range a sector
  /// names. Behind a lock because serving a range advances the reader, and a session snapshot is shared.
  pub geometry: Mutex<LevelGeomSource<InMemoryChunkDataSource>>,
  /// The sector packed by the last `open_sector`, so reading its bytes serves that pack rather than packing again.
  pub packed: Session<PackedSector>,
}

impl SelectedLevel {
  /// What the viewer is shown of the open level.
  pub fn describe(&self) -> SelectedLevelDescription {
    let level: &LevelFile = &self.level;

    SelectedLevelDescription {
      bounds: SectorOutline::merge_bounds(&self.outlines),
      drawables: self.visuals.count_drawable() as u32,
      has_sun: level.lights.as_ref().is_some_and(|it| it.get_sun().is_some()),
      lights: level.lights.as_ref().map_or(0, |it| it.lights.len()) as u32,
      portals: level.portals.as_ref().map_or(0, |it| it.portals.len()) as u32,
      sectors: self.outlines.clone(),
      shader_entries: level.shaders.as_ref().map_or(0, |it| it.entries.len()) as u32,
      source: self.source.clone(),
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
