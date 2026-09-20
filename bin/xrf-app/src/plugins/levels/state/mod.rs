//! What the level plugin holds between calls, one type to a file.

pub(crate) mod level_entry;
pub(crate) mod level_source;
pub(crate) mod level_state;
pub(crate) mod level_sun_description;
pub(crate) mod level_texture_reference;
pub(crate) mod packed_sector;
pub(crate) mod selected_level;
pub(crate) mod selected_level_description;

pub(crate) use level_entry::LevelEntry;
pub(crate) use level_source::LevelSource;
pub(crate) use level_state::LevelState;
pub(crate) use level_texture_reference::LevelTextureReference;
pub(crate) use packed_sector::PackedSector;
pub(crate) use selected_level::SelectedLevel;
pub(crate) use selected_level_description::SelectedLevelDescription;

/// The file every compiled level has, which is the bundle itself.
pub const LEVEL_FILE: &str = "level";

/// The render geometry beside it, which the packed ranges are read out of.
pub const GEOMETRY_FILE: &str = "level.geom";

/// The directory every installation keeps its levels under.
pub const LEVELS_DIRECTORY: &str = "levels";
