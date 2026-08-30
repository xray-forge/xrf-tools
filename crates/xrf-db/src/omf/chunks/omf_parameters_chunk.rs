use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::OmfFile;
use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
use crate::data::ogf::ogf_part::OgfPart;

#[derive(Debug)]
pub struct OmfParametersChunk {
  pub version: u16,
  pub parts: Vec<OgfPart>,
  pub motions: Vec<OgfMotionDefinition>,
}

impl OmfParametersChunk {
  pub const CHUNK_ID: u32 = 15; // 0x14, 0xF

  pub fn get_bones_count(&self) -> usize {
    self.parts.iter().map(|it| it.bones.len()).sum()
  }
}

impl ChunkReadWrite for OmfParametersChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u16 = reader.read_u16::<T>()?;

    if !OmfFile::SUPPORTED_VERSIONS.contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected parameters version {} on read, only versions {:?} are implemented",
        version,
        OmfFile::SUPPORTED_VERSIONS
      )));
    }

    let parts: Vec<OgfPart> = reader
      .read_xr_list::<T, _>()
      .map_err(|error| XrfError::new_read_error(format!("Failed to read ogf parts: {}", error)))?;

    let motions: Vec<OgfMotionDefinition> = OgfMotionDefinition::read_list::<T, _>(reader, version)
      .map_err(|error| XrfError::new_read_error(format!("Failed to read ogf motion definitions: {}", error)))?;

    reader.assert_read("Expect all data to be read from omf parameters chunk")?;

    Ok(Self {
      version,
      parts,
      motions,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if !OmfFile::SUPPORTED_VERSIONS.contains(&self.version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected parameters version {} on write, only versions {:?} are implemented",
        self.version,
        OmfFile::SUPPORTED_VERSIONS
      )));
    }

    writer.write_u16::<T>(self.version)?;

    writer
      .write_xr_list::<T, _>(&self.parts)
      .map_err(|error| XrfError::new_serialization_error(format!("Failed to write ogf parts: {error}")))?;

    OgfMotionDefinition::write_list::<T>(writer, &self.motions, self.version)
      .map_err(|error| XrfError::new_serialization_error(format!("Failed to write ogf motion definitions: {error}")))?;

    Ok(())
  }
}
