use serde::Serialize;
use xrf_engine_target::XrayEngineResolution;
use xrf_vfs::XrayRoots;
use xrf_visual::{SectorOutline, VisualBounds};

use crate::level::LevelSource;
use crate::level::LevelSunDescription;

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
  /// The compiler version the header records, which decides how the level's chunks are laid out.
  pub xrlc_version: u16,
  /// The quality the compiler built the level at, as its header records it.
  pub xrlc_quality: u16,
  /// Visuals the level declares, the hierarchy's inner nodes with them.
  pub visuals: u32,
  /// Visuals that draw something themselves, the hierarchy's leaves.
  pub drawables: u32,
  /// Entries of the shader table, each a shader and the textures it binds.
  pub shader_entries: u32,
  /// Portals joining the level's sectors.
  pub portals: u32,
  /// Lights the level places, its sun among them.
  pub lights: u32,
  /// Whether one of its lights is the sun.
  pub has_sun: bool,
  /// The directional light the level names its sun, which is what its baked occlusion was computed for.
  pub sun: Option<LevelSunDescription>,
  /// What each sector is and where, as the visuals declare them.
  pub sectors: Vec<SectorOutline>,
  /// Extent every sector together covers, which is where a camera is framed from.
  pub bounds: Option<VisualBounds>,
}
