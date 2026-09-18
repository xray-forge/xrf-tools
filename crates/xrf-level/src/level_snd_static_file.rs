use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_snd_static_sound::SndStaticSound;

/// The sounds a level plants, `level.snd_static`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSndStaticFile {
  pub sounds: Vec<SndStaticSound>,
}

impl LevelSndStaticFile {
  /// Reads the sounds from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a sound list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level static sounds were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads the sounds from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a sound list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a record's chunk does not hold exactly one payload.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut sounds: Vec<SndStaticSound> = Vec::new();

    for mut chunk in reader.read_children()? {
      let mut data: ChunkReader<D> = chunk.read_child_by_index(SndStaticSound::DATA_CHUNK_ID)?;

      sounds.push(SndStaticSound::read::<T, D>(&mut data)?);

      data.assert_read("Expect all data to be read from level static sound chunk")?;
      chunk.assert_read("Expect level static sound record to hold one chunk")?;
    }

    Ok(Self { sounds })
  }

  /// Writes the sounds back in the layout the engine reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, sound) in self.sounds.iter().enumerate() {
      let mut data: ChunkWriter = ChunkWriter::new();

      sound.write::<T>(&mut data)?;

      let mut record: ChunkWriter = ChunkWriter::new();

      data.flush_chunk_into::<T>(&mut record.buffer, SndStaticSound::DATA_CHUNK_ID)?;
      record.flush_chunk_into::<T>(&mut writer.buffer, index as u32)?;
    }

    Ok(())
  }
}

impl LevelSndStaticFile {
  /// Sounds that only play inside a window of the day, which four of the 670 shipped ones do.
  pub fn get_scheduled_count(&self) -> usize {
    self.sounds.iter().filter(|sound| sound.active.is_bounded()).count()
  }
}
