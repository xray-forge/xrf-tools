//! A volume set merged into one name table, and the policy for reading an entry out of it.

mod archive_located_entry;
mod archive_open_volume;
mod archive_open_volumes;
mod archive_project;
mod archive_read_policy;
mod archive_read_policy_constants;
mod archive_read_result;
mod archive_volume_discovery;

pub use archive_open_volumes::ArchiveOpenVolumes;
pub use archive_project::ArchiveProject;
pub use archive_read_policy::ArchiveReadPolicy;
pub use archive_read_result::ArchiveReadResult;
