use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, read_u32_chunk};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, to_format_size};

use crate::data::skeleton::skeleton_motion::SkeletonMotion;

#[derive(Debug)]
pub struct SkeletonMotionsChunk {
  pub motions: Vec<SkeletonMotion>,
}

impl SkeletonMotionsChunk {
  pub const CHUNK_ID: u32 = 14; // 0x1A, 0xE
}

impl ChunkReadWrite for SkeletonMotionsChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading motions chunk: {} bytes", reader.read_bytes_remain());

    let mut chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let (count_chunk, motion_chunks): (&mut ChunkReader<D>, &mut [ChunkReader<D>]) = chunks
      .split_first_mut()
      .ok_or_else(|| XrfError::new_read_error("Skeleton motions chunk has no count definition"))?;

    let bones_motions_count: u32 = read_u32_chunk::<T, _>(count_chunk)?;

    assert_equal(
      bones_motions_count as usize,
      motion_chunks.len(),
      "Expect matching skeleton motions chunks count and count definition",
    )?;

    let mut motions: Vec<SkeletonMotion> = Vec::new();

    for chunk in motion_chunks {
      motions.push(chunk.read_xr::<T, _>()?);
    }

    assert!(
      reader.is_ended(),
      "Expect skeleton motions chunk to be ended, {} remain",
      reader.read_bytes_remain()
    );

    Ok(Self { motions })
  }

  /// Write motions as nested chunks, where leading chunk 0 stores motions count and
  /// following chunks 1..=N store motions themselves.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut count_writer: ChunkWriter = ChunkWriter::new();

    count_writer.write_u32::<T>(to_format_size(self.motions.len(), "skeleton motions")?)?;
    writer.write_all(count_writer.flush_chunk_into_buffer::<T>(0)?.as_slice())?;

    for (index, motion) in self.motions.iter().enumerate() {
      let mut motion_writer: ChunkWriter = ChunkWriter::new();

      motion.write::<T>(&mut motion_writer)?;

      writer.write_all(
        motion_writer
          .flush_chunk_into_buffer::<T>(to_format_size(index + 1, "skeleton motion chunk id")?)?
          .as_slice(),
      )?;
    }

    log::info!(
      "Written motions chunk, {} bytes, {} motions",
      writer.bytes_written(),
      self.motions.len()
    );

    Ok(())
  }
}
