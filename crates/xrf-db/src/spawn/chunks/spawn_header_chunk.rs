use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, Section};
use xrf_utils::open_export_file;

use crate::export::FileImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnHeaderChunk {
  pub version: u32,
  pub guid: Uuid,
  pub graph_guid: Uuid,
  pub objects_count: u32,
  pub levels_count: u32,
}

impl SpawnHeaderChunk {
  pub const CHUNK_ID: u32 = 0;
}

impl ChunkReadWrite for SpawnHeaderChunk {
  /// Read header chunk by position descriptor.
  /// Parses binary data into header chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Parsing header chunk, {} bytes", reader.read_bytes_remain());

    let header: Self = Self {
      version: reader.read_u32::<T>()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
      graph_guid: Uuid::from_u128(reader.read_u128::<T>()?),
      objects_count: reader.read_u32::<T>()?,
      levels_count: reader.read_u32::<T>()?,
    };

    reader.assert_read("Expect header chunk to be ended")?;

    Ok(header)
  }

  /// Write header data into chunk writer.
  /// Writes header data in binary format.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u128::<T>(self.guid.as_u128())?;
    writer.write_u128::<T>(self.graph_guid.as_u128())?;
    writer.write_u32::<T>(self.objects_count)?;
    writer.write_u32::<T>(self.levels_count)?;

    log::info!("Written header chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for SpawnHeaderChunk {
  /// Import header data from provided path.
  /// Parse ltx files and populate spawn file.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let ltx: Ltx = Ltx::read_from_path(path.as_ref().join("header.ltx"))?;
    let section: &Section = ltx
      .section("header")
      .expect("Patrol section 'header' should be defined in ltx file");

    Ok(Self {
      version: read_ltx_field("version", section)?,
      guid: read_ltx_field("guid", section)?,
      graph_guid: read_ltx_field("graph_guid", section)?,
      objects_count: read_ltx_field("objects", section)?,
      levels_count: read_ltx_field("level_count", section)?,
    })
  }

  /// Export header data into provided path.
  /// Creates ltx file config with header chunk description.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    ltx
      .with_section("header")
      .set("version", self.version.to_string())
      .set("guid", self.guid.to_string())
      .set("graph_guid", self.graph_guid.to_string())
      .set("objects", self.objects_count.to_string())
      .set("level_count", self.levels_count.to_string());

    ltx.write_to(&mut open_export_file(path.as_ref().join("header.ltx"))?)?;

    log::info!("Exported header chunk");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use uuid::{Uuid, uuid};
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_directory,
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::export::FileImportExport;
  use crate::spawn::chunks::spawn_header_chunk::SpawnHeaderChunk;

  #[test]
  fn test_read_empty() -> XrfResult {
    // A container holding one child that declares no payload: id 0, size 0. Reading a spawn header out of that must
    // fail rather than hand back a default-shaped one.
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0; 8])?.read_child_by_index(0)?;

    let original: XrfResult<SpawnHeaderChunk> = SpawnHeaderChunk::read::<XRayByteOrder, _>(&mut reader);

    assert!(original.is_err(), "Expected failure with empty chunk");

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: SpawnHeaderChunk = SpawnHeaderChunk {
      version: 20,
      guid: Uuid::from_u128(2u128.pow(127)),
      graph_guid: Uuid::from_u128(2u128.pow(64)),
      objects_count: 5050,
      levels_count: 12,
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 44);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 44);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 52);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(SpawnHeaderChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: SpawnHeaderChunk = SpawnHeaderChunk {
      version: 10,
      guid: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
      graph_guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0d9"),
      objects_count: 550,
      levels_count: 12,
    };

    let export_directory: &Path =
      &build_absolute_generated_test_resource_path(&build_relative_test_sample_file_directory(file!()));
    std::fs::create_dir_all(export_directory)?;

    original.export(&export_directory)?;

    assert_eq!(SpawnHeaderChunk::import(&export_directory)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: SpawnHeaderChunk = SpawnHeaderChunk {
      version: 12,
      guid: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
      graph_guid: uuid!("67e55023-10b1-426f-9247-bb680e5fe0c8"),
      objects_count: 6432,
      levels_count: 31,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<SpawnHeaderChunk>(&serialized)?);

    Ok(())
  }
}
