use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{FileImportExport, Ltx, META_TYPE_FIELD, Section};
use xrf_utils::{assert_equal, format_path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesHeaderChunk {
  pub version: u16,
}

impl ParticlesHeaderChunk {
  pub const META_TYPE: &'static str = "particles_header";
  pub const CHUNK_ID: u32 = 1;
}

impl ChunkReadWrite for ParticlesHeaderChunk {
  /// Read version chunk by position descriptor.
  /// Parses binary data into version chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let header_chunk: Self = Self {
      version: reader.read_u16::<T>()?,
    };

    log::info!("Read header chunk, {} bytes", reader.read_bytes_len());

    if header_chunk.version != 1 {
      return Err(XrfError::new_not_implemented_error(
        "Unknown version in particles header chunk, expected v1 only",
      ));
    }

    reader.assert_read("Expect version chunk to be ended")?;

    Ok(header_chunk)
  }

  /// Write particle header into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.version)?;

    log::info!("Written header chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for ParticlesHeaderChunk {
  /// Import header data from provided path.
  /// Parse ltx files and populate spawn file.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    log::info!("Importing particles header: {}", format_path(path.as_ref()));

    let ltx: Ltx = Ltx::read_from_path(path.as_ref().join("header.ltx"))?;
    let section: &Section = ltx
      .section("header")
      .expect("Patrol section 'header' should be defined in ltx file");

    let meta_type: String = section.read_field(META_TYPE_FIELD)?;
    let header_chunk: Self = Self {
      version: section.read_field("version")?,
    };

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expect type metadata to be set correctly",
    )?;
    assert_equal(header_chunk.version, 1, "Expect version chunk to be 1")?;

    Ok(header_chunk)
  }

  /// Export header data into provided path.
  /// Creates ltx file config with header chunk description.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    ltx
      .with_section("header")
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set("version", self.version.to_string());

    ltx.write_to_path(path.as_ref().join("header.ltx"))?;

    log::info!("Exported header chunk");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::FileImportExport;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;

  #[test]
  fn test_read_write_incorrect() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_incorrect.chunk");

    let original: ParticlesHeaderChunk = ParticlesHeaderChunk { version: 2 };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 2);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 2);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 2 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(
      ParticlesHeaderChunk::read::<XRayByteOrder, _>(&mut reader)
        .unwrap_err()
        .to_string(),
      "Not implemented error: Unknown version in particles header chunk, expected v1 only",
    );

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: ParticlesHeaderChunk = ParticlesHeaderChunk { version: 1 };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 2);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 2);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 2 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(ParticlesHeaderChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: ParticlesHeaderChunk = ParticlesHeaderChunk { version: 1 };

    let export_directory: &Path = &GeneratedResources::sample_root(file!());
    std::fs::create_dir_all(export_directory)?;

    original.export(&export_directory)?;

    let read: ParticlesHeaderChunk = ParticlesHeaderChunk::import(&export_directory)?;

    assert_eq!(read, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticlesHeaderChunk = ParticlesHeaderChunk { version: 1 };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<ParticlesHeaderChunk>(&serialized)?);

    Ok(())
  }
}
