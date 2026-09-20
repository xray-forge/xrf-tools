use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section, read_ltx_field};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphHeader {
  pub version: u8,
  pub vertices_count: u16,
  pub edges_count: u32,
  pub points_count: u32,
  pub guid: Uuid,
  pub levels_count: u8,
}

impl ChunkReadWrite for GraphHeader {
  /// Read header data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u8()?,
      vertices_count: reader.read_u16::<T>()?,
      edges_count: reader.read_u32::<T>()?,
      points_count: reader.read_u32::<T>()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
      levels_count: reader.read_u8()?,
    })
  }

  /// Write graph edge data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.version)?;
    writer.write_u16::<T>(self.vertices_count)?;
    writer.write_u32::<T>(self.edges_count)?;
    writer.write_u32::<T>(self.points_count)?;
    writer.write_u128::<T>(self.guid.as_u128())?;
    writer.write_u8(self.levels_count)?;

    Ok(())
  }
}

impl LtxImportExport for GraphHeader {
  /// Import graph header from ltx file.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Graph section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      version: read_ltx_field("version", section)?,
      vertices_count: read_ltx_field("vertex_count", section)?,
      edges_count: read_ltx_field("edges_count", section)?,
      points_count: read_ltx_field("point_count", section)?,
      levels_count: read_ltx_field("level_count", section)?,
      guid: read_ltx_field("guid", section)?,
    })
  }

  /// Export graph header data into level ltx.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("version", self.version.to_string())
      .set("vertex_count", self.vertices_count.to_string())
      .set("edges_count", self.edges_count.to_string())
      .set("point_count", self.points_count.to_string())
      .set("level_count", self.levels_count.to_string())
      .set("guid", self.guid.to_string());

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
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::graph::graph_header::GraphHeader;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: GraphHeader = GraphHeader {
      version: 16,
      vertices_count: 4000,
      edges_count: 230_250,
      points_count: 600_500,
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      levels_count: 5,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 28);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 28);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 28 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphHeader::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: GraphHeader = GraphHeader {
      version: 16,
      vertices_count: 6434,
      edges_count: 456,
      points_count: 5635,
      levels_count: 25,
      guid: uuid!("23e55044-10b1-426f-9247-bb680e5fe0c8"),
    };

    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_generated_test_resource_as_file(config_path.to_str().expect("Valid path"))?;
    let mut ltx: Ltx = Ltx::new();

    original.export("header", &mut ltx)?;
    ltx.write_to(&mut file)?;

    assert_eq!(
      GraphHeader::import("header", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphHeader = GraphHeader {
      version: 12,
      vertices_count: 2341,
      edges_count: 12513,
      points_count: 43231,
      levels_count: 31,
      guid: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<GraphHeader>(&serialized)?);

    Ok(())
  }
}
