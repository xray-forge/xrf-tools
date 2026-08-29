use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::XrfResult;
use xrf_utils::{assert, assert_length, format_path, open_export_file};

use crate::export::FileImportExport;
use crate::particles::chunks::particles_effects_chunk::ParticlesEffectsChunk;
use crate::particles::chunks::particles_firstgen_chunk::ParticlesFirstgenChunk;
use crate::particles::chunks::particles_groups_chunk::ParticlesGroupsChunk;
use crate::particles::chunks::particles_header_chunk::ParticlesHeaderChunk;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesFile {
  pub header: ParticlesHeaderChunk,
  pub effects: ParticlesEffectsChunk,
  pub groups: ParticlesGroupsChunk,
}

impl ParticlesFile {
  /// Read particles from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path)?)
  }

  /// Read particles from file.
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

  /// Read particles from chunks.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    assert(
      !chunks.iter().any(|it| it.id == ParticlesFirstgenChunk::CHUNK_ID),
      "Unexpected first-gen chunk in particles file, unpacking not implemented",
    )?;
    assert_length(chunks, 3, "Unexpected chunks in particles file root")?;

    Ok(Self {
      header: find_required_chunk_by_id(chunks, ParticlesHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      effects: find_required_chunk_by_id(chunks, ParticlesEffectsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      groups: find_required_chunk_by_id(chunks, ParticlesGroupsChunk::CHUNK_ID)?.read_xr::<T, _>()?,
    })
  }

  /// Write particles file data to the file by provided path.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    fs::create_dir_all(path.as_ref().parent().expect("Parent directory"))?;

    self.write_to::<T>(&mut open_export_file(path)?)
  }

  /// Write particles file data to the writer.
  pub fn write_to<T: ByteOrder>(&self, writer: &mut dyn Write) -> XrfResult {
    log::info!(
      "Writing particles file: version {}, {} effects, {} groups",
      self.header.version,
      self.effects.effects.len(),
      self.groups.groups.len(),
    );

    let mut header_chunk_writer: ChunkWriter = ChunkWriter::new();
    header_chunk_writer.write_xr::<T, _>(&self.header)?;
    header_chunk_writer.flush_chunk_into::<T>(writer, ParticlesHeaderChunk::CHUNK_ID)?;

    let mut effects_chunk_writer: ChunkWriter = ChunkWriter::new();
    effects_chunk_writer.write_xr::<T, _>(&self.effects)?;
    effects_chunk_writer.flush_chunk_into::<T>(writer, ParticlesEffectsChunk::CHUNK_ID)?;

    let mut group_chunk_writer: ChunkWriter = ChunkWriter::new();
    group_chunk_writer.write_xr::<T, _>(&self.groups)?;
    group_chunk_writer.flush_chunk_into::<T>(writer, ParticlesGroupsChunk::CHUNK_ID)?;

    Ok(())
  }

  /// Read spawn file from provided path.
  pub fn import_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    log::info!("Importing particles file: {}", format_path(path.as_ref()));

    Ok(Self {
      header: ParticlesHeaderChunk::import(path)?,
      effects: ParticlesEffectsChunk::import(path)?,
      groups: ParticlesGroupsChunk::import(path)?,
    })
  }

  /// Export unpacked ALife spawn file into provided path.
  pub fn export_to_path<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    fs::create_dir_all(path)?;

    self.header.export(path)?;
    self.effects.export(path)?;
    self.groups.export(path)?;

    Ok(())
  }
}
