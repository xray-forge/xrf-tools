use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::{assert_length, to_format_size};

#[derive(Clone, Debug, PartialEq)]
pub struct OgfMotionMark {
  pub name: String,
  pub intervals: Vec<(f32, f32)>,
}

impl OgfMotionMark {
  /// The two-byte line terminator of an empty name, and the interval count.
  pub const MIN_SERIALIZED_SIZE: u64 = 2 + 4;

  /// One interval is a pair of floats.
  pub const MIN_INTERVAL_SIZE: u64 = 4 + 4;
}

impl ChunkReadWrite for OgfMotionMark {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: String = reader.read_w1251_rn_string()?;

    let count: u32 = reader.read_u32::<T>()?;
    let mut intervals: Vec<(f32, f32)> =
      reader.new_bounded_vec(count.into(), Self::MIN_INTERVAL_SIZE, "ogf motion mark intervals")?;

    for _ in 0..count {
      intervals.push((reader.read_f32::<T>()?, reader.read_f32::<T>()?));
    }

    assert_length(
      &intervals,
      count as usize,
      "Expected correct count of OGF mark intervals to be read",
    )?;

    Ok(Self { name, intervals })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_rn_string(&self.name)?;
    writer.write_u32::<T>(to_format_size(self.intervals.len(), "ogf motion mark intervals")?)?;

    for (from, to) in &self.intervals {
      writer.write_f32::<T>(*from)?;
      writer.write_f32::<T>(*to)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::ogf::ogf_motion_mark::OgfMotionMark;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: OgfMotionMark = OgfMotionMark {
      name: String::from("Left"),
      intervals: vec![(0.25, 0.75), (1.5, 2.0)],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    // 4 name bytes + 2 terminator bytes + 4 count bytes + 4 interval floats.
    assert_eq!(writer.bytes_written(), 4 + 2 + 4 + 16);

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(OgfMotionMark::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_without_intervals() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_without_intervals.chunk");

    let original: OgfMotionMark = OgfMotionMark {
      name: String::from("Right"),
      intervals: Vec::new(),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(OgfMotionMark::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn rejects_interval_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    // An empty name and its terminator, then a count no payload can satisfy.
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[13, 10, 255, 255, 255, 255])?;

    let error: String = OgfMotionMark::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared interval count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf motion mark intervals declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
