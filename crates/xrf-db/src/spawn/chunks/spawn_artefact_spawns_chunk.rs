use std::fmt;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{assert_length, open_export_file};

use crate::data::artefact_spawn::artefact_spawn_point::ArtefactSpawnPoint;
use crate::export::{FileImportExport, LtxImportExport};

/// Artefacts spawns samples.
/// Is single plain chunk with nodes list in it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnArtefactSpawnsChunk {
  pub nodes: Vec<ArtefactSpawnPoint>,
}

impl SpawnArtefactSpawnsChunk {
  pub const CHUNK_ID: u32 = 2;
}

impl ChunkReadWrite for SpawnArtefactSpawnsChunk {
  /// Read header chunk by position descriptor.
  /// Parses binary data into artefact spawns chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading artefacts spawns chunk: {} bytes", reader.read_bytes_remain());

    let count: u32 = reader.read_u32::<T>()?;
    let mut nodes: Vec<ArtefactSpawnPoint> = reader.new_bounded_vec(
      count.into(),
      ArtefactSpawnPoint::MIN_SERIALIZED_SIZE,
      "artefact spawn points",
    )?;

    // Parsing CLevelPoint structure, 20 bytes per one.
    for _ in 0..count {
      nodes.push(ArtefactSpawnPoint::read::<T, _>(reader)?);
    }

    assert_length(&nodes, count as usize, "Expected defined count of nodes to be read")?;
    reader.assert_read("Expect artefact spawns chunk to be ended")?;

    Ok(Self { nodes })
  }

  /// Write artefact spawns into chunk writer.
  /// Writes artefact spawns data in binary format.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.nodes.len() as u32)?;

    for node in &self.nodes {
      node.write::<T>(writer)?;
    }

    log::info!("Written artefact spawns chunk, {} bytes", writer.bytes_written());

    Ok(())
  }
}

impl FileImportExport for SpawnArtefactSpawnsChunk {
  /// Import artefact spawns data from provided path.
  /// Parse ltx files and populate spawn file.
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let ltx: Ltx = Ltx::read_from_path(path.as_ref().join("artefact_spawns.ltx"))?;
    let mut nodes: Vec<ArtefactSpawnPoint> = Vec::with_capacity(ltx.sections.len());

    for (name, _) in &ltx.sections {
      nodes.push(ArtefactSpawnPoint::import(name, &ltx)?);
    }

    log::info!("Imported artefact spawns chunk");

    Ok(Self { nodes })
  }

  /// Export artefact spawns data into provided path.
  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    for (index, spawn_point) in self.nodes.iter().enumerate() {
      spawn_point.export(&index.to_string(), &mut ltx)?;
    }

    ltx.write_to(&mut open_export_file(path.as_ref().join("artefact_spawns.ltx"))?)?;

    log::info!("Exported artefact spawns chunk");

    Ok(())
  }
}

impl fmt::Debug for SpawnArtefactSpawnsChunk {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      formatter,
      "ArtefactSpawnsChunk {{ nodes: Vector[{}] }}",
      self.nodes.len()
    )
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

  use crate::data::artefact_spawn::artefact_spawn_point::ArtefactSpawnPoint;
  use crate::data::generic::vector_3d::Vector3d;
  use crate::spawn::chunks::spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: SpawnArtefactSpawnsChunk = SpawnArtefactSpawnsChunk {
      nodes: vec![
        ArtefactSpawnPoint {
          position: Vector3d::new(55.5, 44.4, -33.3),
          level_vertex_id: 255,
          distance: 450.30,
        },
        ArtefactSpawnPoint {
          position: Vector3d::new(-21.0, 13.5, -4.0),
          level_vertex_id: 13,
          distance: 25.11,
        },
      ],
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 44);

    let bytes_written: usize = writer
      .flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename).unwrap(), 0)?;

    assert_eq!(bytes_written, 44);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 44 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(
      SpawnArtefactSpawnsChunk::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn rejects_point_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255, 255, 255])?;

    let error: String = SpawnArtefactSpawnsChunk::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared point count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("artefact spawn points declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
