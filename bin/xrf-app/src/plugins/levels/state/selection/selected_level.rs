use std::sync::{Arc, OnceLock};

use xrf_chunk::InMemoryChunkDataSource;
use xrf_level::{LevelFile, LevelGeomSource, LevelSector, LevelVisualsChunk};
use xrf_material::XraySurfaceDescriptor;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::level_spawn::LevelSpawn;
use crate::plugins::levels::state::packed_details::PackedDetails;
use crate::plugins::levels::state::packed_sectors::PackedSectors;
use crate::plugins::levels::state::selection::level_sun_description::LevelSunDescription;
use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::state::selection::selected_level_description::SelectedLevelDescription;

/// The level a viewer has open, and everything a later call reads without opening it again.
pub struct SelectedLevel {
  pub source: LevelSource,
  /// What each texture reference the level's surfaces bind came to, decided at open.
  pub textures: Vec<LevelTextureReference>,
  /// How the renderer draws each entry of the shader table, in its order, decided at open.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// The roots the level was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  /// What each sector is and where, taken at open from what the visuals declare, so a viewer can decide what to
  /// stream before reading any geometry.
  pub outlines: Vec<SectorOutline>,
  /// Render geometry with its payloads still on the heap where they were read, serving whichever range a sector
  /// names. Shared rather than locked: a range is read without moving the source, so sectors pack side by side.
  pub geometry: LevelGeomSource<InMemoryChunkDataSource>,
  /// The sectors packed by an `open_sector` and not yet served, so reading their bytes serves the pack the read
  /// was described rather than packing again. One entry per read, because reads overlap.
  pub packed: PackedSectors,
  /// The grass packed by an `open_details` and not yet served.
  pub details: PackedDetails,
  /// What the game spawns on the level, read the first time anything asks and kept, a failure with it.
  pub spawn: OnceLock<Result<Arc<LevelSpawn>, String>>,
}

impl SelectedLevel {
  /// What the viewer is shown of the open level.
  pub fn describe(&self) -> SelectedLevelDescription {
    let level: &LevelFile = &self.level;

    SelectedLevelDescription {
      bounds: SectorOutline::merge_bounds(&self.outlines),
      drawables: self.visuals.count_drawable() as u32,
      has_sun: level.lights.as_ref().is_some_and(|it| it.get_sun().is_some()),
      sun: LevelSunDescription::of(level.lights.as_ref().and_then(|it| it.get_sun())),
      lights: level.lights.as_ref().map_or(0, |it| it.lights.len()) as u32,
      portals: level.portals.as_ref().map_or(0, |it| it.portals.len()) as u32,
      roots: self.roots.clone(),
      sectors: self.outlines.clone(),
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
