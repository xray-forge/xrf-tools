use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One triangle of a sound environment mesh: a `CDB::TRI` whose packed word names the environment on either side
/// rather than a game material (`CSoundRender_Scene::set_geometry_env`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSndEnvFace {
  /// Its corners, by index into the mesh's vertices.
  pub vertices: [u32; 3],
  /// The environment in front of the face, by index into the file's names: the low half of the word. The level
  /// editor writes a volume's outer environment here.
  pub front: u16,
  /// The environment behind it, the high half: the volume's inner environment.
  pub back: u16,
}

impl LevelSndEnvFace {
  /// Bytes one face occupies: three vertex indices and the packed word.
  pub const SERIALIZED_SIZE: u64 = 16;
}

impl ChunkReadWrite for LevelSndEnvFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let vertices: [u32; 3] = [
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
    ];
    let packed: u32 = reader.read_u32::<T>()?;

    Ok(Self {
      vertices,
      front: (packed & 0xFFFF) as u16,
      back: (packed >> 16) as u16,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32::<T>(vertex)?;
    }

    writer.write_u32::<T>(u32::from(self.front) | (u32::from(self.back) << 16))?;

    Ok(())
  }
}
