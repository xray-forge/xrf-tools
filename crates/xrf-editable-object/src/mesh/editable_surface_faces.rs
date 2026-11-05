use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

/// The faces of a mesh one surface covers, one entry of `EMESH_CHUNK_SFACE`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSurfaceFaces {
  /// The object surface, by name.
  pub surface: String,
  /// Indices into the mesh's faces, in the order stored; the SDK sorts them on load and writes them back sorted.
  pub faces: Vec<u32>,
}

impl EditableSurfaceFaces {
  /// An empty name and no faces.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 4;
}

impl ChunkReadWrite for EditableSurfaceFaces {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let surface: String = reader.read_w1251_string()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut faces: Vec<u32> = reader.new_bounded_vec(count.into(), 4, "surface faces")?;

    for _ in 0..count {
      faces.push(reader.read_u32::<T>()?);
    }

    Ok(Self { surface, faces })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.surface)?;
    writer.write_u32::<T>(to_format_size(self.faces.len(), "Surface faces")?)?;

    for face in &self.faces {
      writer.write_u32::<T>(*face)?;
    }

    Ok(())
  }
}
