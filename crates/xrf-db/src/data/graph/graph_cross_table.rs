use std::fs::File;
use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkSizePackedIterator, ChunkWriter,
  InMemoryChunkDataSource,
};
use xrf_error::XrfResult;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphCrossTable {
  pub version: u32,
  pub nodes_count: u32,
  pub vertices_count: u32,
  pub level_guid: Uuid,
  pub game_guid: Uuid,
  #[serde(skip_serializing, default)] // Does not make sense for JSON.
  pub data: Vec<u8>,
}

// todo: Import/export list functionality?
impl GraphCrossTable {
  /// Export cross-tables as separate gct chunk file.
  pub fn import_list<T: ByteOrder>(file: &mut File) -> XrfResult<Vec<Self>> {
    let mut cross_tables: Vec<Self> = Vec::new();

    for cross_table_reader in ChunkSizePackedIterator::from_current(&mut ChunkReader::from_file(file.try_clone()?)?) {
      let mut cross_table_reader: ChunkReader<InMemoryChunkDataSource> = cross_table_reader?;

      cross_tables.push(cross_table_reader.read_xr::<T, _>()?);
      cross_table_reader.assert_read("Expect cross table chunk to be ended")?;
    }

    Ok(cross_tables)
  }

  /// Export cross-tables as separate gct chunk file.
  pub fn export_list<T: ByteOrder>(cross_tables: &[Self], file: &mut File) -> XrfResult {
    let mut cross_tables_writer: ChunkWriter = ChunkWriter::new();

    for cross_table in cross_tables {
      let mut cross_table_writer: ChunkWriter = ChunkWriter::new();
      cross_table_writer.write_xr::<T, _>(cross_table)?;

      cross_tables_writer.write_u32::<T>(cross_table_writer.bytes_written() as u32 + 4)?;
      cross_tables_writer.write_all(&cross_table_writer.flush_raw_into_buffer()?)?;
    }

    cross_tables_writer.flush_raw_into(file)?;

    Ok(())
  }
}

impl ChunkReadWriteList for GraphCrossTable {
  /// Read cross tables list data from the chunk.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut cross_tables: Vec<Self> = Vec::new();

    for cross_table_reader in ChunkSizePackedIterator::from_current(reader) {
      let mut cross_table_reader: ChunkReader<D> = cross_table_reader?;

      cross_tables.push(cross_table_reader.read_xr::<T, _>()?);
      cross_table_reader.assert_read("Expect cross table chunk to be ended")?;
    }

    Ok(cross_tables)
  }

  /// Write cross tables list data into the writer.
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, cross_tables: &[Self]) -> XrfResult {
    for table in cross_tables {
      let mut table_writer: ChunkWriter = ChunkWriter::new();
      table_writer.write_xr::<T, _>(table)?;

      writer.write_u32::<T>(table_writer.bytes_written() as u32 + 4)?;
      writer.write_all(&table_writer.buffer)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for GraphCrossTable {
  /// Read cross table data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      nodes_count: reader.read_u32::<T>()?,
      vertices_count: reader.read_u32::<T>()?,
      level_guid: Uuid::from_u128(reader.read_u128::<T>()?),
      game_guid: Uuid::from_u128(reader.read_u128::<T>()?),
      data: reader.read_bytes(reader.read_bytes_remain() as usize)?,
    })
  }

  /// Write cross table data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.nodes_count)?;
    writer.write_u32::<T>(self.vertices_count)?;
    writer.write_u128::<T>(self.level_guid.as_u128())?;
    writer.write_u128::<T>(self.game_guid.as_u128())?;
    writer.write_all(&self.data)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use uuid::uuid;
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_file, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::graph::graph_cross_table::GraphCrossTable;

  #[test]
  fn test_read_write_list() -> XrfResult {
    let filename: String = String::from("read_write_list.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: Vec<GraphCrossTable> = vec![
      GraphCrossTable {
        version: 16,
        nodes_count: 51,
        vertices_count: 35,
        level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
      },
      GraphCrossTable {
        version: 16,
        nodes_count: 24,
        vertices_count: 345,
        level_guid: uuid!("cce55023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("dde55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3, 4, 5, 6],
      },
      GraphCrossTable {
        version: 16,
        nodes_count: 24,
        vertices_count: 345,
        level_guid: uuid!("aa125023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("bbe55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3],
      },
    ];

    GraphCrossTable::write_list::<XRayByteOrder>(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 166);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 166);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 166 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphCrossTable::read_list::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: GraphCrossTable = GraphCrossTable {
      version: 16,
      nodes_count: 51,
      vertices_count: 4000,
      level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 55);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 55);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 55 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphCrossTable::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.gct");
    let mut file: File = overwrite_generated_test_resource_as_file(config_path.to_str().expect("Valid path"))?;

    let original: Vec<GraphCrossTable> = vec![
      GraphCrossTable {
        version: 16,
        nodes_count: 23,
        vertices_count: 62,
        level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
      },
      GraphCrossTable {
        version: 16,
        nodes_count: 22,
        vertices_count: 52,
        level_guid: uuid!("cce55023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("dde55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3, 4, 5, 6],
      },
      GraphCrossTable {
        version: 16,
        nodes_count: 45,
        vertices_count: 637,
        level_guid: uuid!("aa125023-10b1-426f-9247-bb680e5fe0b7"),
        game_guid: uuid!("bbe55023-10b1-426f-9247-bb680e5fe0b7"),
        data: vec![0, 1, 2, 3],
      },
    ];

    GraphCrossTable::export_list::<XRayByteOrder>(&original, &mut file)?;

    assert_eq!(
      GraphCrossTable::import_list::<XRayByteOrder>(&mut open_generated_test_resource_as_file(
        config_path.to_str().unwrap()
      )?,)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphCrossTable = GraphCrossTable {
      version: 24,
      nodes_count: 436,
      vertices_count: 26324,
      level_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      game_guid: uuid!("89e55024-10b1-426f-9247-bb680e5fe0b8"),
      data: vec![],
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<GraphCrossTable>(&serialized)?);

    Ok(())
  }
}
