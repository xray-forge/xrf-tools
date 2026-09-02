use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_length, to_format_size};

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonPart {
  pub name: String,
  pub bones: Vec<(String, u32)>, // name + index.
}

impl SkeletonPart {
  pub fn get_bones(&self) -> Vec<&str> {
    self.bones.iter().map(|it| it.0.as_str()).collect::<Vec<_>>()
  }
}

impl SkeletonPart {
  /// The terminator of an empty name and the bone count.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 2;

  /// One bone is a terminated name and its index.
  pub const MIN_BONE_SIZE: u64 = 1 + 4;
}

impl ChunkReadWriteList for SkeletonPart {
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let count: u16 = reader.read_u16::<T>()?;
    let mut parts: Vec<Self> = reader.new_bounded_vec(count.into(), Self::MIN_SERIALIZED_SIZE, "skeleton parts")?;

    for _ in 0..count {
      parts.push(
        Self::read::<T, _>(reader)
          .map_err(|error| XrfError::new_read_error(format!("Failed to read skeleton part: {error}")))?,
      );
    }

    assert_length(
      &parts,
      count as usize,
      "Expected correct count of skeleton parts to be read",
    )?;

    Ok(parts)
  }

  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, parts: &[Self]) -> XrfResult {
    writer.write_u16::<T>(to_format_size(parts.len(), "skeleton parts")?)?;

    for part in parts {
      part
        .write::<T>(writer)
        .map_err(|error| XrfError::new_serialization_error(format!("Failed to write skeleton part: {error}")))?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for SkeletonPart {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let count: u16 = reader.read_u16::<T>()?;

    let mut bones: Vec<(String, u32)> =
      reader.new_bounded_vec(count.into(), Self::MIN_BONE_SIZE, "skeleton part bones")?;

    for _ in 0..count {
      bones.push((reader.read_w1251_string()?, reader.read_u32::<T>()?));
    }

    Ok(Self { name, bones })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_u16::<T>(to_format_size(self.bones.len(), "skeleton part bones")?)?;

    for (name, index) in &self.bones {
      writer.write_w1251_string(name)?;
      writer.write_u32::<T>(*index)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{
    ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::skeleton::skeleton_part::SkeletonPart;

  #[test]
  fn rejects_a_part_list_past_its_count_field() {
    let parts: Vec<SkeletonPart> = vec![
      SkeletonPart {
        name: String::new(),
        bones: Vec::new(),
      };
      65536
    ];

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      SkeletonPart::write_list::<XRayByteOrder>(&mut writer, &parts)
        .expect_err("expect the part count to exceed its format field")
        .to_string(),
      "Invalid error: skeleton parts exceeds the u16 format limit"
    );
  }

  #[test]
  fn rejects_a_bone_list_past_its_count_field() {
    let part: SkeletonPart = SkeletonPart {
      name: String::from("part"),
      bones: vec![(String::new(), 0); 65536],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      part
        .write::<XRayByteOrder>(&mut writer)
        .expect_err("expect the bone count to exceed its format field")
        .to_string(),
      "Invalid error: skeleton part bones exceeds the u16 format limit"
    );
  }

  #[test]
  fn test_read_write_list() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_list.chunk");

    let original: Vec<SkeletonPart> = vec![
      SkeletonPart {
        name: String::from("default"),
        bones: vec![(String::from("bip01"), 0)],
      },
      SkeletonPart {
        name: String::from("right_hand"),
        bones: vec![(String::from("r_hand"), 1), (String::from("lead_gun"), 2)],
      },
    ];

    SkeletonPart::write_list::<XRayByteOrder>(&mut writer, &original)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(SkeletonPart::read_list::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn rejects_part_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255])?;

    let error: String = SkeletonPart::read_list::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared part count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("skeleton parts declares 65535 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn rejects_part_bone_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    // An empty part name, then a bone count no payload can satisfy.
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 255, 255])?;

    let error: String = SkeletonPart::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared bone count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("skeleton part bones declares 65535 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
