//! What an open level reads beside its own bundle, each kept as a part of its session with what it was read from, so a
//! change to that moves the part and not the level. One part type to a file, owning how its value is read and kept,
//! its value beside it: read from the level's own files (`level`), from the game's spawn and what it names (`spawn`),
//! for what the renderer draws beside the sectors (`drawn`), from the game's environment (`environment`), and from the
//! game's console defaults (`console`). Code names a part by its type's `SessionPart::NAME`; each file's `LEVEL_*_PART`
//! constant is that name as the generated bindings carry it.

mod console;
mod drawn;
mod environment;
mod level;
mod spawn;

pub(crate) use console::level_console_part::LevelConsolePart;
pub(crate) use drawn::level_details_part::LevelDetailsPart;
pub(crate) use drawn::level_lights_part::LevelLightsPart;
pub(crate) use drawn::level_object_motions_part::LevelObjectMotionsPart;
pub(crate) use drawn::level_particles_part::LevelParticlesPart;
pub(crate) use drawn::packed_level_details::PackedLevelDetails;
pub(crate) use drawn::packed_level_lights::PackedLevelLights;
pub(crate) use drawn::packed_level_particles::PackedLevelParticles;
pub(crate) use environment::level_environment::LevelEnvironment;
pub(crate) use environment::level_environment_part::LevelEnvironmentPart;
pub(crate) use environment::level_weather_part::LevelWeatherPart;
pub(crate) use level::level_collision_part::LevelCollisionPart;
pub(crate) use level::level_geometry_part::LevelGeometryPart;
pub use level::level_map_area::LevelMapArea;
pub use level::level_map_description::LevelMapDescription;
pub(crate) use level::level_map_part::LevelMapPart;
pub(crate) use level::level_surfaces_part::LevelSurfacesPart;
pub use level::level_texture_reference::LevelTextureReference;
pub(crate) use level::level_textures_part::LevelTexturesPart;
pub(crate) use spawn::level_graph_part::LevelGraphPart;
pub(crate) use spawn::level_hemi_part::LevelHemiPart;
pub(crate) use spawn::level_sections_part::LevelSectionsPart;
pub(crate) use spawn::level_spawn::LevelSpawn;
pub use spawn::level_spawn_object_hemi::LevelSpawnObjectHemi;
pub(crate) use spawn::level_spawn_part::LevelSpawnPart;
pub(crate) use spawn::level_spawn_visual::LevelSpawnVisual;
pub(crate) use spawn::level_spawn_visuals::LevelSpawnVisuals;
pub(crate) use spawn::level_spawn_visuals_part::LevelSpawnVisualsPart;
pub use spawn::level_start::LevelStart;
pub use spawn::level_start_origin::LevelStartOrigin;
pub(crate) use spawn::level_start_part::LevelStartPart;
pub(crate) use spawn::spawn_visual_reader::SpawnVisualReader;
