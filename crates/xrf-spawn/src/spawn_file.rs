use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::FileImportExport;
use xrf_utils::{assert_equal, assert_length, open_export_file};

use crate::chunks::spawn_alife_spawns_chunk::SpawnALifeSpawnsChunk;
use crate::chunks::spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk;
use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
use crate::chunks::spawn_patrols_chunk::SpawnPatrolsChunk;

/// Descriptor of generic spawn file used by xray game engine.
///
/// Root level samples by ID:
/// 0 - header
/// 1 - alife spawns
/// 2 - alife objects
/// 3 - patrols
/// 4 - game graphs
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnFile {
  pub header: SpawnHeaderChunk,
  pub alife_spawn: SpawnALifeSpawnsChunk,
  pub artefact_spawn: SpawnArtefactSpawnsChunk,
  pub patrols: SpawnPatrolsChunk,
  pub graphs: SpawnGraphsChunk,
}

impl SpawnFile {
  /// Read spawn file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path)?)
  }

  /// Read spawn file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The route an archived entry takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  /// Read spawn file from chunks.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    assert_length(chunks, 5, "Unexpected chunks count in spawn file root, expected 5")?;

    let spawn_file: Self = {
      Self {
        header: find_required_chunk_by_id(chunks, SpawnHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        alife_spawn: find_required_chunk_by_id(chunks, SpawnALifeSpawnsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        artefact_spawn: find_required_chunk_by_id(chunks, SpawnArtefactSpawnsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        patrols: find_required_chunk_by_id(chunks, SpawnPatrolsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
        graphs: find_required_chunk_by_id(chunks, SpawnGraphsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      }
    };

    assert_length(
      &spawn_file.alife_spawn.objects,
      spawn_file.header.objects_count as usize,
      "Expected correct objects count",
    )?;
    assert_equal(
      spawn_file.header.levels_count,
      spawn_file.graphs.header.levels_count as u32,
      "Expected correct levels count",
    )?;

    Ok(spawn_file)
  }

  /// Read only the game graphs chunk of the spawn file by provided path.
  ///
  /// Unlike [`Self::read_from_path`], ALife, artefact spawn and patrol chunks are not parsed at all.
  /// Consumers that only need the level roster stay readable on spawn files containing ALife object
  /// classes without a CLSID mapping, and skip the bulk of the file.
  pub fn read_graphs_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<SpawnGraphsChunk> {
    Self::read_graphs_from_file::<T>(File::open(path)?)
  }

  /// Read only the game graphs chunk of the spawn file from file.
  pub fn read_graphs_from_file<T: ByteOrder>(file: File) -> XrfResult<SpawnGraphsChunk> {
    Self::read_graphs_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Read only the game graphs chunk from a reader over any data source.
  ///
  /// The route an archived spawn file takes: a volume holds no file to slice, only bytes.
  pub fn read_graphs_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<SpawnGraphsChunk> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_graphs_from_chunks::<T, _>(&chunks)
  }

  /// Read only the game graphs chunk of the spawn file from chunks.
  pub fn read_graphs_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<SpawnGraphsChunk> {
    find_required_chunk_by_id(chunks, SpawnGraphsChunk::CHUNK_ID)?.read_xr::<T, _>()
  }

  /// Write spawn file data to the file by provided path.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let path_ref: &Path = path.as_ref();

    if let Some(parent) = path_ref.parent() {
      fs::create_dir_all(parent)?;
      self.write_to::<T>(&mut open_export_file(path)?)
    } else {
      Err(XrfError::new_not_found_error(format!(
        "Spawn file parent directory was not found for {:?}",
        path_ref.to_str()
      )))
    }
  }

  /// Write spawn file data to the writer.
  pub fn write_to<T: ByteOrder>(&self, writer: &mut dyn Write) -> XrfResult {
    let mut header_chunk_writer: ChunkWriter = ChunkWriter::new();
    header_chunk_writer.write_xr::<T, _>(&self.header)?;
    header_chunk_writer.flush_chunk_into::<T>(writer, SpawnHeaderChunk::CHUNK_ID)?;

    let mut alife_spawn_chunk_writer: ChunkWriter = ChunkWriter::new();
    alife_spawn_chunk_writer.write_xr::<T, _>(&self.alife_spawn)?;
    alife_spawn_chunk_writer.flush_chunk_into::<T>(writer, SpawnALifeSpawnsChunk::CHUNK_ID)?;

    let mut artefact_spawn_chunk_writer: ChunkWriter = ChunkWriter::new();
    artefact_spawn_chunk_writer.write_xr::<T, _>(&self.artefact_spawn)?;
    artefact_spawn_chunk_writer.flush_chunk_into::<T>(writer, SpawnArtefactSpawnsChunk::CHUNK_ID)?;

    let mut patrols_chunk_writer: ChunkWriter = ChunkWriter::new();
    patrols_chunk_writer.write_xr::<T, _>(&self.patrols)?;
    patrols_chunk_writer.flush_chunk_into::<T>(writer, SpawnPatrolsChunk::CHUNK_ID)?;

    let mut graphs_chunk_writer: ChunkWriter = ChunkWriter::new();
    graphs_chunk_writer.write_xr::<T, _>(&self.graphs)?;
    graphs_chunk_writer.flush_chunk_into::<T>(writer, SpawnGraphsChunk::CHUNK_ID)?;

    Ok(())
  }

  /// Read spawn file from provided path.
  pub fn import_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Ok(Self {
      header: SpawnHeaderChunk::import(path)?,
      alife_spawn: SpawnALifeSpawnsChunk::import(path)?,
      artefact_spawn: SpawnArtefactSpawnsChunk::import(path)?,
      patrols: SpawnPatrolsChunk::import(path)?,
      graphs: SpawnGraphsChunk::import(path)?,
    })
  }

  /// Export unpacked ALife spawn file into provided path.
  pub fn export_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    fs::create_dir_all(path)?;

    self.header.export(path)?;
    self.alife_spawn.export(path)?;
    self.artefact_spawn.export(path)?;
    self.patrols.export(path)?;
    self.graphs.export(path)?;

    Ok(())
  }
}
