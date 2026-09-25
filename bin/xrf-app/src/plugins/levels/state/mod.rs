//! What the level plugin holds between calls, one type to a file.

pub(crate) mod level_entry;
pub(crate) mod level_source;
pub(crate) mod level_state;
pub(crate) mod packed_details;
pub(crate) mod packed_sector;
pub(crate) mod packed_sectors;
pub(crate) mod selection;

pub(crate) use level_entry::LevelEntry;
pub(crate) use level_source::LevelSource;
pub(crate) use level_state::LevelState;
pub(crate) use packed_details::PackedDetails;
pub(crate) use packed_sector::PackedSector;
pub(crate) use packed_sectors::PackedSectors;
pub(crate) use selection::level_details_description::LevelDetailsDescription;
pub(crate) use selection::level_texture_reference::LevelTextureReference;
pub(crate) use selection::selected_level::SelectedLevel;
pub(crate) use selection::selected_level_description::SelectedLevelDescription;

/// The file every compiled level has, which is the bundle itself.
pub const LEVEL_FILE: &str = "level";

/// The render geometry beside it, which the packed ranges are read out of.
pub const GEOMETRY_FILE: &str = "level.geom";

/// The detail library and planting grid, which the grass is planted from.
pub const DETAILS_FILE: &str = "level.details";

/// The collision form, which the grass is planted onto.
pub const COLLISION_FILE: &str = "level.cform";

/// The directory every installation keeps its levels under.
pub const LEVELS_DIRECTORY: &str = "levels";
