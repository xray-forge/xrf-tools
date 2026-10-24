//! The two formats that share `.spawn`, said in words.

mod archive_level_spawn_description;
mod archive_level_spawn_section;
mod archive_spawn_description;
mod archive_spawn_section;

pub use archive_level_spawn_description::ArchiveLevelSpawnDescription;
pub use archive_spawn_description::ArchiveSpawnDescription;
