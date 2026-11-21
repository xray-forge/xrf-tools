use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Sphere};

#[derive(Debug)]
pub struct OgfHeaderChunk {
  pub version: u8,
  pub model_type: u8,
  pub shader_id: u16,
  pub bounding_box: Aabb,
  pub bounding_sphere: Sphere,
}

impl OgfHeaderChunk {
  pub const CHUNK_ID: u32 = 1;
}

impl ChunkReadWrite for OgfHeaderChunk {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u8 = reader.read_u8()?;

    if version != 4 {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected version '{version}' of OGF file, only version 4 is supported"
      )));
    }

    let header: Self = Self {
      version,
      model_type: reader.read_u8()?,
      shader_id: reader.read_u16()?,
      bounding_box: reader.read_xr()?,
      bounding_sphere: reader.read_xr()?,
    };

    reader.assert_read("Expect all data to be read from ogf header, {} remain")?;

    Ok(header)
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.version)?;
    writer.write_u8(self.model_type)?;
    writer.write_u16(self.shader_id)?;
    writer.write_xr(&self.bounding_box)?;
    writer.write_xr(&self.bounding_sphere)?;

    Ok(())
  }
}
