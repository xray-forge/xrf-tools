use std::io::Write;

use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::shader_blender::ShaderBlender;
use crate::shader_library_file::ShaderLibraryFile;

/// A `shaders.xr` holding the blenders a test needs, and nothing else.
pub struct ShaderLibraryFixture;

impl ShaderLibraryFixture {
  /// A library defining exactly these blenders, in this order.
  pub fn library<B: Clone + Into<ShaderBlender>>(blenders: &[B]) -> XrfResult<Vec<u8>> {
    let mut chunk: ChunkWriter = ChunkWriter::new();

    for (index, blender) in blenders.iter().enumerate() {
      let mut blender_writer: ChunkWriter = ChunkWriter::new();

      blender.clone().into().write::<XRayByteOrder>(&mut blender_writer)?;
      chunk.write_all(&blender_writer.flush_chunk_into_buffer::<XRayByteOrder>(index as u32)?)?;
    }

    let mut library: ChunkWriter = ChunkWriter::new();

    library.write_all(&chunk.flush_chunk_into_buffer::<XRayByteOrder>(ShaderLibraryFile::BLENDERS_CHUNK_ID)?)?;

    library.flush_raw_into_buffer()
  }

  /// Reads library bytes back, for a test that wants the file rather than the bytes.
  pub fn read(bytes: &[u8]) -> XrfResult<ShaderLibraryFile> {
    ShaderLibraryFile::read_from_chunk(&mut ChunkReader::from_bytes(bytes)?)
  }
}
