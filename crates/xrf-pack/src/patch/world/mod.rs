//! One side of a comparison, mounted the way the engine would see it.

mod archive_patch_checksum;
mod archive_patch_entry_point;
mod archive_patch_role;
mod archive_patch_world;

pub(crate) use archive_patch_checksum::ArchivePatchChecksum;
pub(crate) use archive_patch_role::ArchivePatchRole;
pub(crate) use archive_patch_world::ArchivePatchWorld;
