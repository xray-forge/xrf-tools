use serde::Serialize;
use xrf_engine_target::XrayEngineResolution;
use xrf_vfs::XrayRoots;
use xrf_visual::{SectorOutline, VisualBounds};

use crate::state::level_source::LevelSource;
use crate::state::selection::level_sun_description::LevelSunDescription;

/// What the viewer is showing, paired with where it came from: the level itself, its root. What is read of it from
/// other files - where it starts, its map, its surfaces and their textures - each has a query of its own
/// (`describe_start`, `describe_level_map`, `describe_surfaces`, `describe_textures`), read again apart from it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedLevelDescription {
  pub source: LevelSource,
  /// The roots the level and its textures were resolved in, centred on the level itself.
  pub roots: XrayRoots,
  /// The engine everything engine-dependent about the level follows, and what decided it.
  pub engine: XrayEngineResolution,
  pub xrlc_version: u16,
  pub xrlc_quality: u16,
  pub visuals: u32,
  pub drawables: u32,
  pub shader_entries: u32,
  pub portals: u32,
  pub lights: u32,
  pub has_sun: bool,
  /// The directional light the level names its sun, which is what its baked occlusion was computed for.
  pub sun: Option<LevelSunDescription>,
  pub sectors: Vec<SectorOutline>,
  /// Extent every sector together covers, which is where a camera is framed from.
  pub bounds: Option<VisualBounds>,
}
