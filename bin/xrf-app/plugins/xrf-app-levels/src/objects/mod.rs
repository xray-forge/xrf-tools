//! The level object model: every discrete thing placed in a level, named by a stable id, sorted into kinds, and read
//! once from the level's sources for the Outliner, the Inspector, the map and the renderer's helpers alike.

mod kind;
mod level_object;
mod level_object_id;
mod level_object_light;
mod level_object_sources;
mod level_object_spawn;
mod level_objects_description;
mod level_objects_description_helpers;
mod level_objects_description_placements;
mod path;
mod placement;
mod volume;

pub(crate) use kind::{
  LEVEL_OBJECT_KINDS, LEVEL_SOUND_EMITTERS_HELPER_KIND, LevelObjectGroup, LevelObjectKind, LevelObjectKindEntry,
};
pub(crate) use level_object::LevelObject;
pub(crate) use level_object_id::LevelObjectId;
pub(crate) use level_object_light::LevelObjectLight;
pub(crate) use level_object_sources::LevelObjectSources;
pub(crate) use level_object_spawn::LevelObjectSpawn;
pub(crate) use level_objects_description::LevelObjectsDescription;
pub(crate) use path::{LevelObjectPatrol, LevelObjectPatrolPoint, LevelPatrolLink};
pub(crate) use placement::{LevelObjectPlacement, LevelPlacementDetails, LevelPlacementFile, LevelSoundWindow};
pub(crate) use volume::{LevelObjectShape, LevelObjectVolume, LevelRestriction};
