//! A volume set merged into one name table, and the policy for reading an entry out of it.

mod archive_open_volumes;
mod archive_project;
mod archive_project_read_policy;
mod archive_project_read_result;
mod constants;
mod read;
mod shared_payloads;

pub use archive_open_volumes::ArchiveOpenVolumes;
pub use archive_project::ArchiveProject;
pub use archive_project_read_policy::ArchiveProjectReadPolicy;
pub use archive_project_read_result::ProjectReadResult;
