use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::vector_to_string;

use crate::data::generic::u32_bytes::U32Bytes;
use crate::data::generic::vector_3d::Vector3d;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphVertex {
  pub level_point: Vector3d<f32>,
  pub game_point: Vector3d<f32>,
  pub level_id: u8,
  pub level_vertex_id: u32,
  pub vertex_type: U32Bytes,
  pub edges_offset: u32,
  pub level_points_offset: u32,
  pub edges_count: u8,
  pub level_points_count: u8,
}

impl GraphVertex {
  /// Two points, the level id and its packed vertex id, the vertex type, both offsets, and both counts.
  pub const MIN_SERIALIZED_SIZE: u64 = 12 + 12 + 1 + 3 + 4 + 4 + 4 + 1 + 1;
}

impl ChunkReadWrite for GraphVertex {
  /// Read graph vertex data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      level_point: reader.read_xr::<T, _>()?,
      game_point: reader.read_xr::<T, _>()?,
      level_id: reader.read_u8()?,
      level_vertex_id: reader.read_u24::<T>()?,
      vertex_type: reader.read_xr::<T, U32Bytes>()?,
      edges_offset: reader.read_u32::<T>()?,
      level_points_offset: reader.read_u32::<T>()?,
      edges_count: reader.read_u8()?,
      level_points_count: reader.read_u8()?,
    })
  }

  /// Write graph vertex data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.level_point)?;
    writer.write_xr::<T, _>(&self.game_point)?;
    writer.write_u8(self.level_id)?;
    writer.write_u24::<T>(self.level_vertex_id)?;
    writer.write_xr::<T, U32Bytes>(&self.vertex_type)?;
    writer.write_u32::<T>(self.edges_offset)?;
    writer.write_u32::<T>(self.level_points_offset)?;
    writer.write_u8(self.edges_count)?;
    writer.write_u8(self.level_points_count)?;

    Ok(())
  }
}

impl LtxImportExport for GraphVertex {
  /// Import graph vertex from ltx file.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Graph vertex section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      level_point: read_ltx_field("level_point", section)?,
      game_point: read_ltx_field("game_point", section)?,
      level_id: read_ltx_field("level_id", section)?,
      level_vertex_id: read_ltx_field("level_vertex_id", section)?,
      vertex_type: read_ltx_field("vertex_type", section)?,
      edges_offset: read_ltx_field("edge_offset", section)?,
      level_points_offset: read_ltx_field("level_point_offset", section)?,
      edges_count: read_ltx_field("edge_count", section)?,
      level_points_count: read_ltx_field("level_point_count", section)?,
    })
  }

  /// Export graph vertex data into ltx.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("level_point", self.level_point.to_string())
      .set("game_point", self.game_point.to_string())
      .set("level_id", self.level_id.to_string())
      .set("level_vertex_id", self.level_vertex_id.to_string())
      .set("edge_offset", self.edges_offset.to_string())
      .set("level_point_offset", self.level_points_offset.to_string())
      .set("edge_count", self.edges_count.to_string())
      .set("level_point_count", self.level_points_count.to_string())
      .set(
        "vertex_type",
        vector_to_string(&[
          self.vertex_type.0,
          self.vertex_type.1,
          self.vertex_type.2,
          self.vertex_type.3,
        ]),
      );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
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
  use crate::data::graph::graph_vertex::GraphVertex;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: GraphVertex = GraphVertex {
      level_point: Vector3d::new_mock(),
      game_point: Vector3d::new_mock(),
      level_id: 255,
      level_vertex_id: 4000,
      vertex_type: (1, 2, 3, 4).into(),
      edges_offset: 540,
      level_points_offset: 4000,
      edges_count: 252,
      level_points_count: 253,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 42);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 42);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 42 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphVertex::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: GraphVertex = GraphVertex {
      level_point: Vector3d::new_mock(),
      game_point: Vector3d::new_mock(),
      level_id: 53,
      level_vertex_id: 5462,
      vertex_type: (1, 2, 3, 4).into(),
      edges_offset: 5643,
      level_points_offset: 2134,
      edges_count: 14,
      level_points_count: 63,
    };

    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_generated_test_resource_as_file(config_path.to_str().expect("Valid path"))?;
    let mut ltx: Ltx = Ltx::new();

    original.export("graph_vertex", &mut ltx)?;
    ltx.write_to(&mut file)?;

    assert_eq!(
      GraphVertex::import("graph_vertex", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphVertex = GraphVertex {
      level_point: Vector3d::new_mock(),
      game_point: Vector3d::new_mock(),
      level_id: 213,
      level_vertex_id: 5234,
      vertex_type: (1, 2, 3, 4).into(),
      edges_offset: 3242,
      level_points_offset: 6345,
      edges_count: 211,
      level_points_count: 234,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<GraphVertex>(&serialized)?);

    Ok(())
  }
}
