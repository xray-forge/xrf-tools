//! What the level plugin holds between calls, one type to a file.

pub(crate) mod level_entry;
pub(crate) mod level_environment;
pub(crate) mod level_source;
pub(crate) mod level_spawn;
pub(crate) mod level_spawn_lighting;
pub(crate) mod level_spawn_visual;
pub(crate) mod level_spawn_visuals;
pub(crate) mod level_state;
pub(crate) mod selection;

pub(crate) use level_entry::LevelEntry;
pub(crate) use level_environment::LevelEnvironment;
pub(crate) use level_source::LevelSource;
pub(crate) use level_spawn::LevelSpawn;
pub(crate) use level_spawn_lighting::LevelSpawnLighting;
pub(crate) use level_spawn_visual::LevelSpawnVisual;
pub(crate) use level_spawn_visuals::LevelSpawnVisuals;
pub(crate) use level_state::LevelState;
pub(crate) use selection::level_console_defaults::LevelConsoleDefaults;
pub(crate) use selection::level_spawn_category::LevelSpawnCategory;
pub(crate) use selection::level_spawn_object::LevelSpawnObject;
pub(crate) use selection::level_spawn_object_details::LevelSpawnObjectDetails;
pub(crate) use selection::level_spawn_object_hemi::LevelSpawnObjectHemi;
pub(crate) use selection::level_spawn_objects_description::LevelSpawnObjectsDescription;
pub(crate) use selection::level_spawn_release::LevelSpawnRelease;
pub(crate) use selection::level_start::LevelStart;
pub(crate) use selection::level_start_origin::LevelStartOrigin;
pub(crate) use selection::level_texture_reference::LevelTextureReference;
pub(crate) use selection::level_weather_cycle::LevelWeatherCycle;
pub(crate) use selection::level_weather_description::LevelWeatherDescription;
pub(crate) use selection::selected_level::SelectedLevel;
pub(crate) use selection::selected_level_description::SelectedLevelDescription;

/// The file every compiled level has, which is the bundle itself.
pub const LEVEL_FILE: &str = "level";

/// The render geometry beside it, which the packed ranges are read out of.
pub const GEOMETRY_FILE: &str = "level.geom";

/// The detail library and planting grid, which the grass is planted from.
pub const DETAILS_FILE: &str = "level.details";

/// The collision form, which the grass is planted onto and which says whether a start is under the open sky.
pub const COLLISION_FILE: &str = "level.cform";

/// The particle systems the level plants and where, `CLevel::Load_GameSpecific_After`.
pub const PS_STATIC_FILE: &str = "level.ps_static";

/// The compiled lights, whose point lights light the level's dynamic objects (`CLight_DB::LoadHemi`).
pub const LIGHTS_FILE: &str = "build.lights";

/// The directory every installation keeps its levels under.
pub const LEVELS_DIRECTORY: &str = "levels";
