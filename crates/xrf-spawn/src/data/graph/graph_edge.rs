use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
  pub game_vertex_id: u16,
  pub distance: f32,
}

impl GraphEdge {
  pub const MIN_SERIALIZED_SIZE: u64 = 2 + 4;
}

impl ChunkReadWrite for GraphEdge {
  /// Read edge from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      game_vertex_id: reader.read_u16::<T>()?,
      distance: reader.read_f32::<T>()?,
    })
  }

  /// Write graph edge data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.game_vertex_id)?;
    writer.write_f32::<T>(self.distance)?;

    Ok(())
  }
}

impl LtxImportExport for GraphEdge {
  /// Import graph edge from ltx file.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Graph section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      game_vertex_id: section.read_field("game_vertex_id")?,
      distance: section.read_field("distance")?,
    })
  }

  /// Export graph edge data into ltx.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("game_vertex_id", self.game_vertex_id.to_string())
      .set("distance", self.distance.to_string());

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
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::graph::graph_edge::GraphEdge;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: GraphEdge = GraphEdge {
      game_vertex_id: 713,
      distance: 400.50,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 6);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 6);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 6 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphEdge::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: GraphEdge = GraphEdge {
      game_vertex_id: 352,
      distance: 2554.50,
    };

    let config_path: &Path = &GeneratedResources::sample_path(file!(), "import_export.ltx");
    let mut file: File = GeneratedResources::overwrite(config_path.to_str().expect("Valid path"))?;
    let mut ltx: Ltx = Ltx::new();

    original.export("graph_edge", &mut ltx)?;

    ltx.write_to(&mut file)?;

    assert_eq!(
      GraphEdge::import("graph_edge", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphEdge = GraphEdge {
      game_vertex_id: 713,
      distance: 400.50,
    };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<GraphEdge>(&serialized)?);

    Ok(())
  }
}
