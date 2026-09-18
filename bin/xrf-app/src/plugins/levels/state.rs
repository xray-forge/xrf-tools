use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use xrf_chunk::InMemoryChunkDataSource;
use xrf_level::{LevelFile, LevelGeomSource, LevelSector, LevelVisualsChunk};
use xrf_vfs::XrayRoots;
use xrf_visual::{SectorOutline, SectorPackage, VisualBounds};

use crate::core::session::Session;

pub const LEVELS_DIRECTORY: &str = "levels";
pub const LEVEL_FILE: &str = "level";
pub const GEOMETRY_FILE: &str = "level.geom";

/// One compiled level the roots hold, as a picker lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEntry {
  /// The name the installation knows the level by, which is its directory under `levels`.
  pub name: String,
  /// Its engine identity, which is what opening it takes.
  pub logical_path: String,
  /// Whether `level.geom` sits beside the bundle; a level without it draws nothing.
  pub has_geometry: bool,
}

/// Ownership for the level a viewer has open.
pub struct LevelState {
  pub selected: Session<SelectedLevel>,
}

impl LevelState {
  pub fn new() -> Self {
    Self {
      selected: Session::new("level"),
    }
  }
}

pub struct SelectedLevel {
  pub source: LevelSource,
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
  pub packed: Session<SectorPackage>,
}

/// Where a compiled level is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelSource {
  /// A compiled level directory on disk, named by its filesystem path.
  Directory { path: String },
  /// A level of the mounted roots, named by its engine identity, `levels\<name>`.
  Asset { logical_path: String },
}

impl LevelSource {
  pub fn label(&self) -> &str {
    match self {
      Self::Directory { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// Returns the level's filesystem path when its source provides one.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::Directory { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }
}

/// What the viewer is showing, paired with where it came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedLevelDescription {
  pub source: LevelSource,
  pub xrlc_version: u16,
  pub xrlc_quality: u16,
  pub visuals: u32,
  pub drawables: u32,
  pub shader_entries: u32,
  pub portals: u32,
  pub lights: u32,
  pub has_sun: bool,
  pub sectors: Vec<SectorOutline>,
  /// Extent every sector together covers, which is where a camera is framed from.
  pub bounds: Option<VisualBounds>,
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
      visuals: self.visuals.visuals.len() as u32,
      xrlc_quality: level.header.xrlc_quality,
      xrlc_version: level.header.xrlc_version,
    }
  }
}

/// The sectors a level names, which is what the outlines are taken from.
pub fn sectors_of(level: &LevelFile) -> &[LevelSector] {
  level.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors)
}
