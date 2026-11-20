use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Where a bone sits in its skeleton, the first `BONE_CHUNK_DEF` `CBone::Save` writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableBoneDefinition {
  pub name: String,
  /// The parent bone, by name, empty for the root.
  pub parent: String,
  /// The weight map of the meshes this bone moves.
  pub weight_map: String,
}

impl ChunkReadWrite for EditableBoneDefinition {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: reader.read_w1251_string()?,
      parent: reader.read_w1251_string()?,
      weight_map: reader.read_w1251_string()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_w1251_string(&self.parent)?;
    writer.write_w1251_string(&self.weight_map)?;

    Ok(())
  }
}
