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

pub use kind::{LEVEL_OBJECT_KINDS, LevelObjectGroup, LevelObjectKind, LevelObjectKindEntry};
pub use level_object::LevelObject;
pub use level_object_id::LevelObjectId;
pub use level_object_light::LevelObjectLight;
pub use level_object_sources::LevelObjectSources;
pub use level_object_spawn::LevelObjectSpawn;
pub use level_objects_description::LevelObjectsDescription;
pub use path::{LevelObjectPatrol, LevelObjectPatrolPoint, LevelPatrolLink};
pub use placement::{LevelObjectPlacement, LevelPlacementDetails, LevelPlacementFile};
pub use volume::{LevelObjectShape, LevelObjectVolume, LevelRestriction};
