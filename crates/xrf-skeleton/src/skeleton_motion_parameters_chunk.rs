use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::skeleton_motion_definition::SkeletonMotionDefinition;
use crate::skeleton_part::SkeletonPart;

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

  pub fn get_bones_count(&self) -> usize {
    self.parts.iter().map(|it| it.bones.len()).sum()
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
