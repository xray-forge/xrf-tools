use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

/// One bone part of a skeleton, `SBonePart` as `EOBJ_CHUNK_BONEPARTS2` stores it: a named group of bones a motion
/// can play on alone, the legs apart from the torso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableBonePart {
  pub alias: String,
  /// The bones in the part, by name.
  pub bones: Vec<String>,
}

impl EditableBonePart {
  /// An empty alias and no bones.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 4;
}

impl ChunkReadWrite for EditableBonePart {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let alias: String = reader.read_w1251_string()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut bones: Vec<String> = reader.new_bounded_vec(count.into(), 1, "bone part bones")?;

    for _ in 0..count {
      bones.push(reader.read_w1251_string()?);
    }

    Ok(Self { alias, bones })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.alias)?;
    writer.write_u32::<T>(to_format_size(self.bones.len(), "Bone part bones")?)?;

    for bone in &self.bones {
      writer.write_w1251_string(bone)?;
    }

    Ok(())
  }
}
