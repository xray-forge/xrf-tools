//! What a level's spawn draws and says of its objects: the category each is drawn in, the objects the renderer stands,
//! one object's details, and why a new game releases one.

mod level_spawn_category;
mod level_spawn_object_details;
mod level_spawn_release;
mod level_spawn_render;

pub use level_spawn_category::LevelSpawnCategory;
pub use level_spawn_object_details::LevelSpawnObjectDetails;
pub use level_spawn_release::LevelSpawnRelease;
