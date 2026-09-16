use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::sound::sound_environment::SoundEnvironment;

/// The sound environment library, `senvironment.xr`.
///
/// `SoundEnvironment_LIB::Load` (`xrSound/SoundRender_Environment.cpp`) reads it as a flat run of chunks, one preset
/// each. A level's `level.snd_env` names presets from here by their position in the file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundEnvironmentFile {
  pub environments: Vec<SoundEnvironment>,
}

impl SoundEnvironmentFile {
  /// Reads a sound environment library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a sound environment library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Sound environment library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a sound environment library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a sound environment library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound environment library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a preset cannot be read, or one does not fill the chunk holding it.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut environments: Vec<SoundEnvironment> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      let environment: SoundEnvironment = SoundEnvironment::read::<T, D>(&mut chunk)?;

      chunk.assert_read("Expect all data to be read from sound environment chunk")?;

      environments.push(environment);
    }

    Ok(Self { environments })
  }

  /// Writes the library back as the flat run of chunks `SoundEnvironment_LIB::Save` writes.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, environment) in self.environments.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      environment.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, index as u32)?;
    }

    Ok(())
  }

  /// The preset a name resolves to, matched the way `SoundEnvironment_LIB::GetID` matches it.
  pub fn find_environment(&self, name: &str) -> Option<&SoundEnvironment> {
    self
      .environments
      .iter()
      .find(|environment| environment.name.eq_ignore_ascii_case(name))
  }
}
