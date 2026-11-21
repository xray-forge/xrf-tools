use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// One material of the scene, `b_material` (`communicate.h`): a texture under a render and a compiler shader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjMaterial {
  /// The texture, by index into the texture table.
  pub texture: u16,
  /// The render shader, by index into the render shader table.
  pub shader: u16,
  /// The compiler shader, by index into the compiler shader table.
  pub compiler_shader: u16,
  pub sector: u16,
  pub reserved: u16,
  /// The two bytes `#pragma pack(4)` puts before the area, kept so a rewrite gives back what was read.
  pub area_padding: u16,
  pub internal_max_area: u32,
}

impl BuildPrjMaterial {
  /// Bytes one material occupies, padding included.
  pub const SERIALIZED_SIZE: u64 = 6 * 2 + 4;
}

impl ChunkReadWrite for BuildPrjMaterial {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      texture: reader.read_u16()?,
      shader: reader.read_u16()?,
      compiler_shader: reader.read_u16()?,
      sector: reader.read_u16()?,
      reserved: reader.read_u16()?,
      area_padding: reader.read_u16()?,
      internal_max_area: reader.read_u32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in [
      self.texture,
      self.shader,
      self.compiler_shader,
      self.sector,
      self.reserved,
      self.area_padding,
    ] {
      writer.write_u16(value)?;
    }

    writer.write_u32(self.internal_max_area)?;

    Ok(())
  }
}
