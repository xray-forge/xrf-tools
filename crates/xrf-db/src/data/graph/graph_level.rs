use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::data::generic::vector_3d::Vector3d;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

/// `GameGraph::SLevel::load` in xray codebase.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphLevel {
  pub name: String,
  pub offset: Vector3d<f32>,
  pub id: u8,
  pub section: String,
  pub guid: Uuid,
}

impl GraphLevel {
  /// The terminators of an empty name and section, the offset, the id, and the guid.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 12 + 1 + 1 + 16;
}

impl ChunkReadWrite for GraphLevel {
  /// Read graph level data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: reader.read_w1251_string()?,
      offset: reader.read_xr::<T, _>()?,
      id: reader.read_u8()?,
      section: reader.read_w1251_string()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
    })
  }

  /// Write graph level data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;

    writer.write_xr::<T, _>(&self.offset)?;
    writer.write_u8(self.id)?;
    writer.write_w1251_string(&self.section)?;
    writer.write_u128::<T>(self.guid.as_u128())?;

    Ok(())
  }
}

impl LtxImportExport for GraphLevel {
  /// Import patrols data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Graph level section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      name: read_ltx_field("name", section)?,
      offset: read_ltx_field("offset", section)?,
      id: read_ltx_field("id", section)?,
      section: read_ltx_field("section", section)?,
      guid: read_ltx_field("guid", section)?,
    })
  }

  /// Export graph level data into ltx.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("name", &self.name)
      .set("section", &self.section)
      .set("offset", self.offset.to_string())
      .set("id", self.id.to_string())
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
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::graph::graph_level::GraphLevel;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: GraphLevel = GraphLevel {
      id: 255,
      name: String::from("test-level"),
      section: String::from("test-level-section"),
      guid: uuid!("89e55023-10b1-426f-9247-bb680e5fe0b8"),
      offset: Vector3d::new(0.5, 5.55, -1.5),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 59);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 59);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 59 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphLevel::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: GraphLevel = GraphLevel {
      id: 78,
      name: String::from("test-level-exported"),
      section: String::from("test-level-section"),
      guid: uuid!("89e55023-10b1-426f-9247-bb680e5fe0a5"),
      offset: Vector3d::new(0.25, 5.55, -1.5),
    };

    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_generated_test_resource_as_file(config_path.to_str().expect("Valid path"))?;
    let mut ltx: Ltx = Ltx::new();

    original.export("graph_level", &mut ltx)?;
    ltx.write_to(&mut file)?;

    assert_eq!(
      GraphLevel::import("graph_level", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphLevel = GraphLevel {
      id: 243,
      name: String::from("test-level-example"),
      section: String::from("test-level-section"),
      guid: uuid!("89e55023-10b1-426f-9247-bb680e5fe0b8"),
      offset: Vector3d::new(0.25, 5.55, -1.5),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<GraphLevel>(&serialized)?);

    Ok(())
  }
}
