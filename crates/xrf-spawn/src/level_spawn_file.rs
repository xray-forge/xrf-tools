use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_spawn_object::LevelSpawnObject;

/// The objects a level spawns, `level.spawn`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnFile {
  pub objects: Vec<LevelSpawnObject>,
}

impl LevelSpawnFile {
  /// Reads a level spawn list from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a spawn list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level spawn list was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a level spawn list from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a spawn list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a spawn list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk does not open with a spawn packet, which is what tells this format from the set
  /// that shares its extension.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut objects: Vec<LevelSpawnObject> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      let id: u32 = chunk.id;

      objects.push(LevelSpawnObject::read::<T, D>(id, &mut chunk)?);
    }

    Ok(Self { objects })
  }

  /// Writes the list back as the run of packets the engine walks.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for object in &self.objects {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      object.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, object.id)?;
    }

    Ok(())
  }
}

impl LevelSpawnFile {
  /// What the level spawns, counted by the section each object is built from.
  pub fn get_sections(&self) -> BTreeMap<&str, usize> {
    let mut counted: BTreeMap<&str, usize> = BTreeMap::new();

    for object in &self.objects {
      *counted.entry(object.section.as_str()).or_default() += 1;
    }

    counted
  }
}
