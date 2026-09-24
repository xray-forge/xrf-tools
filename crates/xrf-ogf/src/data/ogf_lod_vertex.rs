use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One corner of an impostor facet, `FLOD::_vertex`: where it stands in the level, where it samples the level's
/// impostor atlas, and its baked light.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodVertex {
  pub position: Vector3d,
  pub texture_coordinate: (f32, f32),
  /// `D3DCOLOR`: the baked colour, and the hemisphere term in the alpha byte.
  pub rgb_hemi: u32,
  pub sun: u8,
}

impl OgfLodVertex {
  /// Bytes the struct takes on disk, its compiler's padding included: `FLOD::Load` reads the facets whole.
  pub const SIZE: usize = 28;

  /// Padding after the sun byte, which aligns the struct to four.
  const PADDING: usize = 3;
}

impl ChunkReadWrite for OgfLodVertex {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let vertex: Self = Self {
      position: reader.read_xr::<T, _>()?,
      texture_coordinate: (reader.read_f32::<T>()?, reader.read_f32::<T>()?),
      rgb_hemi: reader.read_u32::<T>()?,
      sun: reader.read_u8()?,
    };

    for _ in 0..Self::PADDING {
      reader.read_u8()?;
    }

    Ok(vertex)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.texture_coordinate.0)?;
    writer.write_f32::<T>(self.texture_coordinate.1)?;
    writer.write_u32::<T>(self.rgb_hemi)?;
    writer.write_u8(self.sun)?;

    for _ in 0..Self::PADDING {
      writer.write_u8(0)?;
    }

    Ok(())
  }
}
