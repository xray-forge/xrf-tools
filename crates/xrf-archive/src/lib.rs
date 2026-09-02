#![doc = include_str!("../README.md")]

mod archive_descriptor;
mod archive_file_descriptor;
mod archive_header;
mod byte_order;
mod constants;
mod file_io;
mod project;
mod reader;
mod volume_readers;

pub use archive_descriptor::ArchiveDescriptor;
pub use archive_file_descriptor::ArchiveFileDescriptor;
pub use constants::{
  CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_DATA, CHUNK_ID_FILE_DESCRIPTORS, CHUNK_ID_METADATA,
  CHUNK_SIZE_FIELD_SIZE, DESCRIPTOR_ROW_FIELDS_SIZE, DESCRIPTOR_ROW_SIZE_FIELD_SIZE,
};
pub use project::{ArchiveProject, ArchiveProjectReadPolicy, ProjectReadResult};
pub use volume_readers::{ArchiveVolumeReaders, write_descriptor_contents};
