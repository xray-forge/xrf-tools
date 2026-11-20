use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{Vector2d, Vector3d};

/// One corner of an impostor facet, `FLOD::_vertex`: where it stands in the level, where it samples the level's
/// impostor atlas, and its baked light.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodVertex {
  pub position: Vector3d,
  pub texture_coordinate: Vector2d,
  /// `D3DCOLOR`: the baked colour, and the hemisphere term in the alpha byte.
  pub rgb_hemi: u32,
  pub sun: u8,
  /// The struct's padding after the sun byte, which aligns it to four. xrLC leaves whatever its memory held there; it
  /// is kept so a rewrite gives back what was read.
  pub padding: [u8; Self::PADDING],
}

impl OgfLodVertex {
  /// Bytes the struct takes on disk, its compiler's padding included: `FLOD::Load` reads the facets whole.
  pub const SIZE: usize = 28;

  /// Padding bytes after the sun byte.
  pub const PADDING: usize = 3;
}

impl ChunkReadWrite for OgfLodVertex {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      texture_coordinate: reader.read_xr::<T, _>()?,
      rgb_hemi: reader.read_u32::<T>()?,
      sun: reader.read_u8()?,
      padding: [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.texture_coordinate)?;
    writer.write_u32::<T>(self.rgb_hemi)?;
    writer.write_u8(self.sun)?;

    for byte in self.padding {
      writer.write_u8(byte)?;
    }

    Ok(())
  }
}
