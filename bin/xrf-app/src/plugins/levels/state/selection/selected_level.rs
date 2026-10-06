use std::sync::{Arc, OnceLock};

use xrf_chunk::InMemoryChunkDataSource;
use xrf_engine_target::XrayEngineResolution;
use xrf_level::{LevelCformTracer, LevelFile, LevelGeomSource, LevelSector, LevelVisualsChunk};
use xrf_ltx::{Ltx, LtxDialect};
use xrf_material::XraySurfaceDescriptor;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::plugins::levels::state::level_environment::LevelEnvironment;
use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::level_spawn::LevelSpawn;
use crate::plugins::levels::state::level_spawn_lighting::LevelSpawnLighting;
use crate::plugins::levels::state::level_spawn_visuals::LevelSpawnVisuals;
use crate::plugins::levels::state::selection::level_start::LevelStart;
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
  /// The rules the game's configs are resolved with, as the open was asked to read them.
  pub dialect: Arc<dyn LtxDialect>,
  /// The engine the game's configs are read as, where the engines read them differently, and what decided it.
  pub engine: XrayEngineResolution,
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  /// What each sector is and where, taken at open from what the visuals declare, so a viewer can decide what to
  /// stream before reading any geometry.
  pub outlines: Vec<SectorOutline>,
  /// Where the level opens, resolved at open; `None` leaves it to the viewer.
  pub start: Option<LevelStart>,
  /// Render geometry with its payloads still on the heap where they were read, serving whichever range a sector
  /// names. Shared rather than locked: a range is read without moving the source, so sectors pack side by side.
  pub geometry: LevelGeomSource<InMemoryChunkDataSource>,
  /// What the game spawns on the level, read the first time anything asks and kept, a failure with it.
  pub spawn: OnceLock<Result<Arc<LevelSpawn>, String>>,
  /// Each visual a spawned object stands as, read the first time anything asks and kept, or why it cannot be.
  pub spawn_visuals: LevelSpawnVisuals,
  /// How the level lights its spawned objects, held while their models are read.
  pub spawn_lighting: LevelSpawnLighting,
  /// The sections of the game's resolved `system.ltx` its spawned objects name, which their lights are read from;
  /// kept likewise.
  pub sections: OnceLock<Result<Arc<Ltx>, String>>,
  /// The game's environment configs and the level's cycles, which its weather is played from; kept likewise.
  pub environment: OnceLock<Result<Arc<LevelEnvironment>, String>>,
  /// The collision form as rays test it, which lights spawned objects and collides particles; kept likewise.
  pub collision: OnceLock<Result<Arc<LevelCformTracer>, String>>,
}

impl SelectedLevel {
  /// What the viewer is shown of the open level.
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
      start: self.start.clone(),
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
