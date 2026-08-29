use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::assert_length;

use crate::data::ogf::ogf_bone::OgfBone;

#[derive(Debug, Serialize, Deserialize)]
pub struct OgfBonesChunk {
  pub bones: Vec<OgfBone>,
}

impl OgfBonesChunk {
  pub const CHUNK_ID: u32 = 13;

  pub fn get_bone_names(&self) -> Vec<&str> {
    self.bones.iter().map(|it| it.name.as_str()).collect::<Vec<_>>()
  }
}

impl ChunkReadWrite for OgfBonesChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading bones chunk: {} bytes", reader.read_bytes_remain());

    let count: u32 = reader.read_u32::<T>()?;
    let mut bones: Vec<OgfBone> = reader.new_bounded_vec(count.into(), OgfBone::MIN_SERIALIZED_SIZE, "ogf bones")?;

    for _ in 0..count {
      bones.push(reader.read_xr::<T, _>()?);
    }

    reader.assert_read("Expect all data to be read from ogf bones chunk")?;
    assert_length(&bones, count as usize, "Expected correct count of bones to be read")?;

    Ok(Self { bones })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.bones.len() as u32)?;

    for bone in &self.bones {
      writer.write_xr::<T, _>(bone)?
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::ogf::chunks::ogf_bones_chunk::OgfBonesChunk;

  #[test]
  fn rejects_bone_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255, 255, 255])?;

    let error: String = OgfBonesChunk::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared bone count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf bones declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
