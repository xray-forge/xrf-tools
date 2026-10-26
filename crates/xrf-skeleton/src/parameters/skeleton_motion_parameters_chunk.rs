use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::parameters::{SkeletonMotionDefinition, SkeletonPart};

#[derive(Debug)]
pub struct SkeletonMotionParametersChunk {
  pub version: u16,
  pub parts: Vec<SkeletonPart>,
  pub motions: Vec<SkeletonMotionDefinition>,
}

impl SkeletonMotionParametersChunk {
  pub const CHUNK_ID: u32 = 15; // 0x14, 0xF

  /// The newest version of this chunk's own `version` field, `xrOGF_SMParamsVersion`. `motions_value::load` reads every
  /// version up to it with one layout, which only marks extend ([`SkeletonMotionDefinition::MARKS_VERSION`]).
  pub const MAXIMUM_VERSION: u16 = 4;

  /// Partitions a bank may declare, `MAX_PARTS` (`SkeletonMotionDefs.hpp:6`): `CPartition` holds a fixed array of four,
  /// which `motions_value::load` would write past for a fifth.
  pub const MAXIMUM_PARTS: usize = 4;

  pub fn get_bones_count(&self) -> usize {
    SkeletonPart::count_bones(&self.parts)
  }

  /// Refuses partitions the engine cannot load: more than [`Self::MAXIMUM_PARTS`], or bone indices that are not each
  /// key run exactly once. `motions_value::load` hands run `i` to the bone stored with index `i` (`rm_bones`), so an index
  /// past the runs writes out of bounds, and a repeated one leaves another run with no bone.
  fn assert_parts(parts: &[SkeletonPart]) -> XrfResult {
    if parts.len() > Self::MAXIMUM_PARTS {
      return Err(XrfError::new_invalid_error(format!(
        "Skeleton motion parameters declare {} parts, the engine holds at most {}",
        parts.len(),
        Self::MAXIMUM_PARTS
      )));
    }

    let runs: usize = SkeletonPart::count_bones(parts);
    let mut is_assigned: Vec<bool> = vec![false; runs];

    for (name, index) in parts.iter().flat_map(|part| &part.bones) {
      let Some(run) = usize::try_from(*index).ok().filter(|run| *run < runs) else {
        return Err(XrfError::new_invalid_error(format!(
          "Skeleton part bone '{name}' is keyed by run {index}, past the {runs} runs its partitions declare"
        )));
      };

      if std::mem::replace(&mut is_assigned[run], true) {
        return Err(XrfError::new_invalid_error(format!(
          "Skeleton part bone '{name}' is keyed by run {index}, which another bone already is"
        )));
      }
    }

    Ok(())
  }

  fn assert_version(version: u16, operation: &str) -> XrfResult {
    if version > Self::MAXIMUM_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected parameters version {version} on {operation}, only versions up to {} are implemented",
        Self::MAXIMUM_VERSION
      )));
    }

    Ok(())
  }
}

impl ChunkReadWrite for SkeletonMotionParametersChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u16 = reader.read_u16::<T>()?;

    Self::assert_version(version, "read")?;

    let parts: Vec<SkeletonPart> = reader
      .read_xr_list::<T, _>()
      .map_err(|error| XrfError::new_read_error(format!("Failed to read skeleton parts: {}", error)))?;

    Self::assert_parts(&parts)?;

    let motions: Vec<SkeletonMotionDefinition> = SkeletonMotionDefinition::read_list::<T, _>(reader, version)
      .map_err(|error| XrfError::new_read_error(format!("Failed to read skeleton motion definitions: {}", error)))?;

    reader.assert_read("Expect all data to be read from skeleton motion parameters chunk")?;

    Ok(Self {
      version,
      parts,
      motions,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    Self::assert_version(self.version, "write")?;
    Self::assert_parts(&self.parts)?;

    writer.write_u16::<T>(self.version)?;

    writer
      .write_xr_list::<T, _>(&self.parts)
      .map_err(|error| XrfError::new_serialization_error(format!("Failed to write skeleton parts: {error}")))?;

    SkeletonMotionDefinition::write_list::<T>(writer, &self.motions, self.version).map_err(|error| {
      XrfError::new_serialization_error(format!("Failed to write skeleton motion definitions: {error}"))
    })?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use byteorder::WriteBytesExt;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::parameters::{SkeletonMotionDefinition, SkeletonMotionParametersChunk, SkeletonPart};

  fn part(name: &str, bones: &[(&str, u32)]) -> SkeletonPart {
    SkeletonPart {
      name: String::from(name),
      bones: bones
        .iter()
        .map(|(bone, index)| (String::from(*bone), *index))
        .collect(),
    }
  }

  fn chunk(parts: Vec<SkeletonPart>) -> SkeletonMotionParametersChunk {
    SkeletonMotionParametersChunk {
      version: SkeletonMotionParametersChunk::MAXIMUM_VERSION,
      parts,
      motions: vec![SkeletonMotionDefinition::new_mock(Vec::new())],
    }
  }

  fn write(chunk: &SkeletonMotionParametersChunk) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    chunk.write::<XRayByteOrder>(&mut writer)?;

    writer.flush_raw_into_buffer()
  }

  /// Writes partitions without the writer's own check, as a tool that does not share it would.
  fn write_unchecked(parts: &[SkeletonPart]) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_u16::<XRayByteOrder>(SkeletonMotionParametersChunk::MAXIMUM_VERSION)?;
    writer.write_xr_list::<XRayByteOrder, _>(parts)?;
    SkeletonMotionDefinition::write_list::<XRayByteOrder>(&mut writer, &[], 4)?;

    writer.flush_raw_into_buffer()
  }

  fn read(bytes: &[u8]) -> XrfResult<SkeletonMotionParametersChunk> {
    SkeletonMotionParametersChunk::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(
      bytes,
    )?)
  }

  #[test]
  fn reads_back_partitions_whose_bones_are_keyed_out_of_partition_order() -> XrfResult {
    // The shape a three part human bank takes: each part lists its bones, and their runs interleave across parts.
    let original: SkeletonMotionParametersChunk = chunk(vec![
      part("legs", &[("bip01", 0), ("bip01_l_thigh", 3)]),
      part("torso", &[("bip01_spine", 1)]),
      part("head", &[("bip01_head", 2)]),
    ]);
    let read: SkeletonMotionParametersChunk = read(&write(&original)?)?;

    assert_eq!(read.parts, original.parts);
    assert_eq!(read.motions, original.motions);
    assert_eq!(read.get_bones_count(), 4);

    Ok(())
  }

  #[test]
  fn refuses_more_parts_than_the_engine_holds() -> XrfResult {
    let parts: Vec<SkeletonPart> = (0..5).map(|index| part("part", &[("bone", index)])).collect();
    let expected: &str = "Invalid error: Skeleton motion parameters declare 5 parts, the engine holds at most 4";

    assert_eq!(read(&write_unchecked(&parts)?).unwrap_err().to_string(), expected);
    assert_eq!(write(&chunk(parts)).unwrap_err().to_string(), expected);

    Ok(())
  }

  #[test]
  fn refuses_a_bone_keyed_by_a_run_the_payload_does_not_hold() -> XrfResult {
    let parts: Vec<SkeletonPart> = vec![part("default", &[("bip01", 0), ("bip01_spine", 2)])];

    assert_eq!(
      read(&write_unchecked(&parts)?).unwrap_err().to_string(),
      "Invalid error: Skeleton part bone 'bip01_spine' is keyed by run 2, past the 2 runs its partitions declare"
    );

    Ok(())
  }

  #[test]
  fn refuses_two_bones_keyed_by_one_run() -> XrfResult {
    let parts: Vec<SkeletonPart> = vec![part("legs", &[("bip01", 0)]), part("torso", &[("bip01_spine", 0)])];

    assert_eq!(
      write(&chunk(parts)).unwrap_err().to_string(),
      "Invalid error: Skeleton part bone 'bip01_spine' is keyed by run 0, which another bone already is"
    );

    Ok(())
  }
}
