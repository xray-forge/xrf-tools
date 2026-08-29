use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::{assert_length, to_format_size};

use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatrolLink {
  pub index: u32,
  pub links: Vec<(u32, f32)>,
}

impl ChunkReadWriteList for PatrolLink {
  /// Read links from chunk file.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut links: Vec<Self> = Vec::new();

    while reader.has_data() {
      links.push(Self::read::<T, _>(reader)?);
    }

    reader.assert_read("Chunk data should be read for patrol links")?;

    Ok(links)
  }

  /// Write list patrol links into chunk writer.
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult {
    for link in list {
      link.write::<T>(writer)?;
    }

    Ok(())
  }
}

impl PatrolLink {
  /// One link is a destination index and its weight.
  pub const MIN_LINK_SIZE: u64 = 4 + 4;
}

impl ChunkReadWrite for PatrolLink {
  /// Read patrol link from chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let index: u32 = reader.read_u32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;

    let mut vertices: Vec<(u32, f32)> = reader.new_bounded_vec(count.into(), Self::MIN_LINK_SIZE, "patrol links")?;

    for _ in 0..count {
      let to: u32 = reader.read_u32::<T>()?; // from->to in u16.
      let weight: f32 = reader.read_f32::<T>()?;

      vertices.push((to, weight));
    }

    assert_length(
      &vertices,
      count as usize,
      "Expected correct count of patrol links to be read",
    )?;

    Ok(Self { index, links: vertices })
  }

  /// Write patrol link data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.index)?;
    writer.write_u32::<T>(to_format_size(self.links.len(), "patrol links")?)?;

    for (to, weight) in &self.links {
      writer.write_u32::<T>(*to)?;
      writer.write_f32::<T>(*weight)?;
    }

    Ok(())
  }
}

impl LtxImportExport for PatrolLink {
  /// Import patrol point link from ltx config.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Patrol point link section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let index: u32 = read_ltx_field("index", section)?;
    let count: usize = read_ltx_field("count", section)?;

    let mut links: Vec<(u32, f32)> = Vec::new();

    for link in 0..count {
      links.push((
        read_ltx_field(&format!("from.{link}"), section)?,
        read_ltx_field(&format!("weight.{link}"), section)?,
      ))
    }

    assert_length(&links, count, "Expected to import exact count of patrol links")?;

    Ok(Self { index, links })
  }

  /// Export patrol link data into ltx.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("index", self.index.to_string())
      .set("count", self.links.len().to_string());

    for (index, (from, weight)) in self.links.iter().enumerate() {
      ltx
        .with_section(section_name)
        .set(format!("from.{index}"), from.to_string())
        .set(format!("weight.{index}"), weight.to_string());
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{
    ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::patrols::patrol_link::PatrolLink;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: PatrolLink = PatrolLink {
      index: 1000,
      links: vec![(10, 1.5), (11, 2.5), (12, 3.5)],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 32);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 32);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 32 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(PatrolLink::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_list() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_list.chunk");

    let original: Vec<PatrolLink> = vec![
      PatrolLink {
        index: 1000,
        links: vec![(10, 1.5), (11, 2.5), (12, 3.5)],
      },
      PatrolLink {
        index: 1001,
        links: vec![(20, 1.5)],
      },
    ];

    PatrolLink::write_list::<XRayByteOrder>(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 48);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 48);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 48 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(PatrolLink::read_list::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: PatrolLink = PatrolLink {
      index: 1000,
      links: vec![(10, 1.5), (11, 2.5), (12, 3.5)],
    };

    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "import_export.ltx");
    let mut file: File = overwrite_file(config_path)?;
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;
    ltx.write_to(&mut file)?;

    assert_eq!(
      PatrolLink::import("data", &Ltx::read_from_path(config_path)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: PatrolLink = PatrolLink {
      index: 1000,
      links: vec![(10, 1.5), (11, 2.5), (12, 3.5)],
    };

    let mut file: File = overwrite_file(build_absolute_generated_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<PatrolLink>(&serialized)?);

    Ok(())
  }

  #[test]
  fn rejects_link_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    // The link index, then a count no payload can satisfy.
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 0, 0, 0, 255, 255, 255, 255])?;

    let error: String = PatrolLink::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared link count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("patrol links declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
