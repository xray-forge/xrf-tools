//! Constants mirrored from the engine so every level rule can be traced to a load-time assertion.
//!
//! Sources are `xray-16` (OpenXRay), which is the engine XRF targets. Vanilla Call of Pripyat is
//! stricter in one place: it accepts only `XRAI_VERSION_CS_COP` AI-maps. The file names are the format readers' own.

use xrf_level::{
  LevelAiFile, LevelAiVersion, LevelCformFile, LevelDetailsFile, LevelFile, LevelGameFile, LevelGeomFile,
};

/// `XRCL_PRODUCTION_VERSION`, asserted by `CLevel::Load` as an exact match.
///
/// `R_ASSERT2(XRCL_PRODUCTION_VERSION == H.XRLC_version, "Incompatible level version.")`
/// - `src/xrEngine/IGame_Level.cpp:116`
pub const LEVEL_PRODUCTION_VERSION: u16 = 14;

/// `CFORM_CURRENT_VERSION`, asserted by `CObjectSpace::Load` as an exact match.
///
/// `R_ASSERT(CFORM_CURRENT_VERSION == H.version)` - `src/xrCDB/xr_area.cpp:149`
pub const CFORM_CURRENT_VERSION: u32 = 4;

/// `XRAI_VERSION_ALLOWED`, the lowest AI-map version the engine loads.
///
/// `ASSERT_XRAI_VERSION_MATCH` - `src/Common/LevelStructure.hpp:593`
pub const AI_VERSION_ALLOWED: u32 = LevelAiVersion::Soc as u32;

/// `XRAI_CURRENT_VERSION`, the highest AI-map version the engine loads.
///
/// `ASSERT_XRAI_VERSION_MATCH` - `src/Common/LevelStructure.hpp:593`
pub const AI_CURRENT_VERSION: u32 = LevelAiVersion::Skyloader as u32;

/// Level configuration describing the map rendered in the UI.
pub const LEVEL_LTX_FILE: &str = "level.ltx";

/// Files every built level bundle contains, verified against 94 bundles across four gamedata trees.
pub const REQUIRED_LEVEL_FILES: [&str; 7] = [
  LevelFile::FILE_NAME,
  LevelCformFile::FILE_NAME,
  LevelGameFile::FILE_NAME,
  LevelGeomFile::FILE_NAME,
  LevelGeomFile::DETAIL_FILE_NAME,
  LEVEL_LTX_FILE,
  "level.spawn",
];

/// Compiled level geometry, shaders and sectors.
pub const LEVEL_FILE: &str = LevelFile::FILE_NAME;

/// Collision form used by the collision database.
pub const LEVEL_CFORM_FILE: &str = LevelCformFile::FILE_NAME;

/// AI-map file, required for every level reachable from the game graph.
pub const LEVEL_AI_FILE: &str = LevelAiFile::FILE_NAME;

/// Detail model description, always shipped together with its compiled texture atlas.
pub const LEVEL_DETAILS_FILE: &str = LevelDetailsFile::FILE_NAME;

/// Compiled detail texture atlas, always shipped together with the detail model description.
pub const LEVEL_DETAILS_TEXTURE_FILE: &str = "build_details.dds";

/// Root directory holding level bundles.
pub const LEVELS_DIRECTORY: &str = "levels";

/// Configuration declaring which levels have a single player map.
pub const SINGLE_PLAYER_MAPS_FILE: &str = "game_maps_single.ltx";

/// Configuration declaring which levels have a multiplayer map.
pub const MULTIPLAYER_MAPS_FILE: &str = "game_maps_mp.ltx";

/// Section listing single player level map declarations.
pub const SINGLE_PLAYER_MAPS_SECTION: &str = "level_maps_single";

/// Section listing multiplayer level map declarations.
pub const MULTIPLAYER_MAPS_SECTION: &str = "level_maps_mp";

/// Section of `level.ltx` describing the level map rendered in the UI.
pub const LEVEL_MAP_SECTION: &str = "level_map";

/// Field of `[level_map]` pointing at the level map texture.
pub const LEVEL_MAP_TEXTURE_FIELD: &str = "texture";
