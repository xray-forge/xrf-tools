#![doc = include_str!("../README.md")]

pub(crate) mod chunk_constants;
pub(crate) mod iterator;
pub(crate) mod read_write;
pub(crate) mod reader;
pub(crate) mod source;
pub(crate) mod writer;
pub(crate) mod xray_byte_order;

pub use crate::chunk_constants::{
  CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_FIELD_SIZE, CHUNK_ID_MASK, CHUNK_SIZE_FIELD_SIZE,
};
pub use crate::iterator::chunk_iterator::ChunkIterator;
pub use crate::iterator::chunk_size_packed_iterator::ChunkSizePackedIterator;
pub use crate::read_write::chunk_read_write::ChunkReadWrite;
pub use crate::read_write::chunk_read_write_list::ChunkReadWriteList;
pub use crate::read_write::chunk_read_write_optional::ChunkReadWriteOptional;
pub use crate::reader::chunk_line::ChunkLine;
pub use crate::reader::chunk_reader::ChunkReader;
pub use crate::reader::chunk_trailing::ChunkTrailing;
pub use crate::source::chunk_data_source::ChunkDataSource;
pub use crate::source::chunk_memory_source::InMemoryChunkDataSource;
pub use crate::writer::chunk_writer::ChunkWriter;
pub use crate::xray_byte_order::XRayByteOrder;
