use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::level::level_shader_entry::{LevelShaderEntry, LevelShaderReference};

/// `fsL_SHADERS` chunk of the `level` file, listing every shader and texture set the level geometry
/// references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelShadersChunk {
  pub entries: Vec<LevelShaderEntry>,
}

impl LevelShadersChunk {
  pub const CHUNK_ID: u32 = 2;

  /// One entry is a terminated string, so an empty one still costs its terminator.
  pub const MIN_ENTRY_SIZE: u64 = 1;

  /// Iterate over entries the renderer actually resolves.
  pub fn references(&self) -> impl Iterator<Item = &LevelShaderReference> {
    self.entries.iter().filter_map(|entry| match entry {
      LevelShaderEntry::Reference(reference) => Some(reference),
      _ => None,
    })
  }

  /// Iterate over entries the renderer cannot resolve without crashing.
  pub fn malformed(&self) -> impl Iterator<Item = &str> {
    self.entries.iter().filter_map(|entry| match entry {
      LevelShaderEntry::Malformed(raw) => Some(raw.as_str()),
      _ => None,
    })
  }
}

impl ChunkReadWrite for LevelShadersChunk {
  /// Read level shaders table from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut entries: Vec<LevelShaderEntry> =
      reader.new_bounded_vec(count.into(), Self::MIN_ENTRY_SIZE, "level shaders")?;

    for _ in 0..count {
      entries.push(LevelShaderEntry::parse(&reader.read_w1251_string()?));
    }

    reader.assert_read("Expect level shaders chunk to be ended")?;

    Ok(Self { entries })
  }

  /// Write level shaders table into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.entries.len() as u32)?;

    for entry in &self.entries {
      writer.write_w1251_string(&entry.to_raw())?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::level::level_shader_entry::{LevelShaderEntry, LevelShaderReference};
  use crate::level::level_shaders_chunk::LevelShadersChunk;

  #[test]
  fn splits_entries_by_resolvability() {
    let chunk: LevelShadersChunk = LevelShadersChunk {
      entries: vec![
        LevelShaderEntry::Empty,
        LevelShaderEntry::Malformed(String::from("broken")),
        LevelShaderEntry::parse("shader/texture"),
      ],
    };

    assert_eq!(chunk.references().count(), 1);
    assert_eq!(chunk.malformed().collect::<Vec<_>>(), vec!["broken"]);
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: LevelShadersChunk = LevelShadersChunk {
      entries: vec![
        LevelShaderEntry::Empty,
        LevelShaderEntry::Malformed(String::from("no_delimiter")),
        LevelShaderEntry::Reference(LevelShaderReference {
          shader: String::from("def_shaders\\def_vertex"),
          textures: vec![String::from("prop\\prop_fence")],
        }),
      ],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      LevelShadersChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(LevelShadersChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn rejects_entry_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255, 255, 255])?;

    let error: String = LevelShadersChunk::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared entry count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("level shaders declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
