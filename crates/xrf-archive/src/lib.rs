#![doc = include_str!("../README.md")]

mod archive_project;
mod archive_read_policy;
mod archive_read_policy_constants;
mod archive_read_result;
mod archive_volume_discovery;
mod payload;
mod volume;

pub use archive_project::ArchiveProject;
pub use archive_read_policy::ArchiveReadPolicy;
pub use archive_read_result::ArchiveReadResult;
pub use payload::{ArchiveOpenVolumes, ArchiveSharedPayload};
pub use volume::{
  ArchiveDescriptor, ArchiveFileDescriptor, CHUNK_ID_DATA, CHUNK_ID_FILE_DESCRIPTORS, CHUNK_ID_METADATA,
  DESCRIPTOR_ROW_FIELDS_SIZE, DESCRIPTOR_ROW_SIZE_FIELD_SIZE,
};
pub use xrf_chunk::{CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK, CHUNK_SIZE_FIELD_SIZE};
