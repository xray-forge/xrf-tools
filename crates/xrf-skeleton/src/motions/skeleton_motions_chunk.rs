use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, to_format_size};

use crate::motions::SkeletonMotion;

#[derive(Debug)]
pub struct SkeletonMotionsChunk {
  pub motions: Vec<SkeletonMotion>,
}

impl SkeletonMotionsChunk {
  pub const CHUNK_ID: u32 = 14; // 0x1A, 0xE

  /// The chunk holding the motion count, ahead of the motions numbered from one.
  pub const COUNT_CHUNK_ID: u32 = 0;

  /// Motions a bank may hold: a motion id carries its index in 14 bits beside a 2-bit slot, which `motions_value::load`
  /// verifies the count against (`xrCore/Animation/SkeletonMotions.cpp`).
  pub const MAXIMUM_MOTIONS: usize = 0x3FFF;

  fn assert_count_fits(count: usize) -> XrfResult {
    if count >= Self::MAXIMUM_MOTIONS {
      return Err(XrfError::new_invalid_error(format!(
        "Skeleton motions chunk holds {count} motions, a motion id addresses fewer than {}",
        Self::MAXIMUM_MOTIONS
      )));
    }

    Ok(())
  }
}

impl ChunkReadWrite for SkeletonMotionsChunk {
  /// Reads the count, then each motion by its id, as `motions_value::load` finds them: chunk `n + 1` is motion `n`.
  ///
  /// # Errors
  ///
  /// Returns an error when the count chunk is not one `u32`, the count is more than a motion id addresses or is not the
  /// number of motion chunks, or a motion's chunk is absent or does not read.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading motions chunk: {} bytes", reader.read_bytes_remain());

    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let count: usize =
      ChunkReader::find_required_by_id(&chunks, Self::COUNT_CHUNK_ID)?.read_u32_payload::<T>()? as usize;

    Self::assert_count_fits(count)?;

    assert_equal(
      count,
      chunks.len().saturating_sub(1),
      "Expect matching skeleton motions chunks count and count definition",
    )?;

    let mut motions: Vec<SkeletonMotion> = Vec::with_capacity(count);

    for index in 1..=count {
      motions.push(
        ChunkReader::find_required_by_id(&chunks, to_format_size(index, "skeleton motion chunk id")?)?
          .read_xr::<T, _>()?,
      );
    }

    reader.assert_read("Expect skeleton motions chunk to be ended")?;

    Ok(Self { motions })
  }

  /// Write motions as nested chunks, where leading chunk 0 stores motions count and
  /// following chunks 1..=N store motions themselves.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    Self::assert_count_fits(self.motions.len())?;

    let mut count_writer: ChunkWriter = ChunkWriter::new();

    count_writer.write_u32::<T>(to_format_size(self.motions.len(), "skeleton motions")?)?;
    writer.write_all(
      count_writer
        .flush_chunk_into_buffer::<T>(Self::COUNT_CHUNK_ID)?
        .as_slice(),
    )?;

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

#[cfg(test)]
mod tests {
  use byteorder::WriteBytesExt;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::motions::{SkeletonMotion, SkeletonMotionsChunk};

  fn new_motion(label: &str) -> SkeletonMotion {
    SkeletonMotion {
      label: String::from(label),
      count: 1,
      keys: vec![0, 1, 2, 3],
    }
  }

  /// Reads a motions chunk whose children are laid out in the order given, each under its own id.
  fn read_children(children: Vec<(u32, ChunkWriter)>) -> XrfResult<SkeletonMotionsChunk> {
    let mut bytes: Vec<u8> = Vec::new();

    for (id, child) in children {
      bytes.extend(child.flush_chunk_into_buffer::<XRayByteOrder>(id)?);
    }

    SkeletonMotionsChunk::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(&bytes)?)
  }

  fn count_chunk(count: u32) -> XrfResult<ChunkWriter> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_u32::<XRayByteOrder>(count)?;

    Ok(writer)
  }

  fn motion_chunk(label: &str) -> XrfResult<ChunkWriter> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    new_motion(label).write::<XRayByteOrder>(&mut writer)?;

    Ok(writer)
  }

  #[test]
  fn reads_each_motion_by_its_id_rather_than_its_position() -> XrfResult {
    // `motions_value::load` finds chunk `n + 1` for motion `n`, so a bank storing them out of order still loads in
    // id order.
    let chunk: SkeletonMotionsChunk = read_children(vec![
      (0, count_chunk(2)?),
      (2, motion_chunk("second")?),
      (1, motion_chunk("first")?),
    ])?;

    assert_eq!(chunk.motions, vec![new_motion("first"), new_motion("second")]);

    Ok(())
  }

  #[test]
  fn refuses_a_bank_missing_a_motion_id() -> XrfResult {
    let error: String = read_children(vec![(0, count_chunk(1)?), (2, motion_chunk("orphan")?)])
      .expect_err("expect a missing motion chunk to be refused")
      .to_string();

    assert!(
      error.contains("Chunk with ID 1 was not found"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn refuses_more_motions_than_a_motion_id_addresses() {
    let chunk: SkeletonMotionsChunk = SkeletonMotionsChunk {
      motions: vec![new_motion("idle"); SkeletonMotionsChunk::MAXIMUM_MOTIONS],
    };

    let error: String = chunk
      .write::<XRayByteOrder>(&mut ChunkWriter::new())
      .expect_err("expect the motion count to exceed a motion id")
      .to_string();

    assert!(
      error.contains("a motion id addresses fewer than 16383"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn writes_the_count_then_motions_numbered_from_one() -> XrfResult {
    let original: SkeletonMotionsChunk = SkeletonMotionsChunk {
      motions: vec![new_motion("first"), new_motion("second")],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(SkeletonMotionsChunk::CHUNK_ID)?;
    let children: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_bytes(&bytes)?
      .read_child_by_index(0)?
      .read_children()?;

    assert_eq!(children.iter().map(|it| it.id).collect::<Vec<u32>>(), [0, 1, 2]);

    Ok(())
  }
}
