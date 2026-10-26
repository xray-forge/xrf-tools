use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::environment::sound_environment::SoundEnvironment;

/// The sound environment library, `senvironment.xr`, `SoundEnvironment_LIB`: presets a level's sound environments
/// reach by name and its compiled geometry by number, their order. By default an empty one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
  /// `SoundEnvironment_LIB::Load` opens the presets by chunk id, 0 upwards, until one is missing, so a chunk past a
  /// gap or repeating an id holds a preset the game never has. Such a chunk is refused rather than listed.
  ///
  /// # Errors
  ///
  /// Returns an error when a preset cannot be read, one does not fill the chunk holding it, or a chunk is one the
  /// engine never reaches.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut environments: Vec<SoundEnvironment> = Vec::with_capacity(chunks.len());

    // Stable, so of two chunks sharing an id the first in the file comes first, as the engine finds it.
    chunks.sort_by_key(|chunk| chunk.id);

    for (expected, mut chunk) in (0u32..).zip(chunks) {
      if chunk.id != expected {
        return Err(XrfError::new_invalid_error(if chunk.id < expected {
          format!(
            "Sound environment library repeats chunk {}, which the engine never loads",
            chunk.id
          )
        } else {
          format!(
            "Sound environment library has no chunk {expected}, so the engine never loads chunk {}",
            chunk.id
          )
        }));
      }

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
  /// Returns an error when the writer refuses the bytes, or the presets outnumber the chunk ids.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, environment) in self.environments.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      environment.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, to_format_size(index, "sound environment chunk")?)?;
    }

    Ok(())
  }

  /// Turns every preset into what `SoundEnvironment_LIB::Save` writes, version 4: see [`SoundEnvironment::normalize`].
  pub fn normalize(&mut self) {
    for environment in &mut self.environments {
      environment.normalize();
    }
  }

  /// `SoundEnvironment_LIB::Get`: the first preset a name resolves to, ignoring ASCII case as `xr_stricmp` does.
  pub fn find_environment(&self, name: &str) -> Option<&SoundEnvironment> {
    self.get_environment_index(name).map(|index| &self.environments[index])
  }

  /// [`Self::find_environment`], for changing the preset it finds.
  pub fn find_environment_mut(&mut self, name: &str) -> Option<&mut SoundEnvironment> {
    self
      .get_environment_index(name)
      .map(|index| &mut self.environments[index])
  }

  /// `SoundEnvironment_LIB::GetID`: the number of the first preset a name resolves to, which is what a compiled level
  /// stores in its sound geometry.
  pub fn get_environment_index(&self, name: &str) -> Option<usize> {
    self
      .environments
      .iter()
      .position(|environment| environment.name.eq_ignore_ascii_case(name))
  }

  /// The SDK's "Append" with no preset selected: a new preset of the EAX defaults under `name`, added last.
  ///
  /// # Errors
  ///
  /// Returns an error for an empty name, or one the library already holds.
  pub fn append_environment(&mut self, name: &str) -> XrfResult<&mut SoundEnvironment> {
    self.push_environment(SoundEnvironment::new(name))
  }

  /// `SoundEnvironment_LIB::Append` from a parent: a copy of the preset named `source` under `name`, added last.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no `source`, or `name` is empty or already taken.
  pub fn clone_environment(&mut self, source: &str, name: &str) -> XrfResult<&mut SoundEnvironment> {
    let mut copy: SoundEnvironment = self
      .find_environment(source)
      .ok_or_else(|| Self::new_missing_error(source))?
      .clone();

    copy.name = name.to_owned();

    self.push_environment(copy)
  }

  /// The SDK's rename: gives the preset named `from` the name `to`.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no `from`, or `to` is empty or names another preset.
  pub fn rename_environment(&mut self, from: &str, to: &str) -> XrfResult {
    let index: usize = self
      .get_environment_index(from)
      .ok_or_else(|| Self::new_missing_error(from))?;

    if self.get_environment_index(to).is_some_and(|other| other != index) {
      return Err(Self::new_taken_error(to));
    }

    Self::require_name(to)?;

    self.environments[index].name = to.to_owned();

    Ok(())
  }

  /// `SoundEnvironment_LIB::Remove`: takes the preset named `name` out, keeping the others in their order.
  ///
  /// Every preset after it moves down a number, so a compiled level built against the library before addresses its
  /// presets after it wrongly until it is built again.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no such preset.
  pub fn remove_environment(&mut self, name: &str) -> XrfResult<SoundEnvironment> {
    let index: usize = self
      .get_environment_index(name)
      .ok_or_else(|| Self::new_missing_error(name))?;

    Ok(self.environments.remove(index))
  }

  fn push_environment(&mut self, environment: SoundEnvironment) -> XrfResult<&mut SoundEnvironment> {
    Self::require_name(&environment.name)?;

    if self.find_environment(&environment.name).is_some() {
      return Err(Self::new_taken_error(&environment.name));
    }

    self.environments.push(environment);

    Ok(self.environments.last_mut().expect("the preset was just added"))
  }

  /// The SDK names a preset before adding it, and a level reaching an unnamed one would match nothing it writes.
  fn require_name(name: &str) -> XrfResult {
    if name.is_empty() {
      return Err(XrfError::new_invalid_error("A sound environment needs a name"));
    }

    Ok(())
  }

  /// The engine looks a preset up by its name ignoring case and takes the first, so a second of one name is
  /// unreachable.
  fn new_taken_error(name: &str) -> XrfError {
    XrfError::new_invalid_error(format!("The sound environment library already holds '{name}'"))
  }

  fn new_missing_error(name: &str) -> XrfError {
    XrfError::new_not_found_error(format!("The sound environment library holds no '{name}'"))
  }
}
