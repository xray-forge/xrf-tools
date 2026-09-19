//! Archive volume structure: header chunks, entry descriptors, and format constants.

mod archive_constants;
mod archive_descriptor;
mod archive_file_descriptor;
mod archive_header;
mod archive_reader;

pub use archive_constants::{
  CHUNK_ID_DATA, CHUNK_ID_FILE_DESCRIPTORS, CHUNK_ID_METADATA, DESCRIPTOR_ROW_FIELDS_SIZE,
  DESCRIPTOR_ROW_SIZE_FIELD_SIZE,
};
pub use archive_descriptor::ArchiveDescriptor;
pub use archive_file_descriptor::ArchiveFileDescriptor;
pub(crate) use archive_reader::ArchiveReader;
