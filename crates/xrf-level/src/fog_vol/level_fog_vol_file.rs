use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, narrow_format_size};

use crate::fog_vol::level_fog_volume::LevelFogVolume;

/// The volumetric fog bodies of a level, `level.fog_vol`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelFogVolFile {
  pub version: u16,
  pub volumes: Vec<LevelFogVolume>,
}

impl LevelFogVolFile {
  /// The file a level's fog volumes are shipped as, in its directory.
  pub const FILE_NAME: &'static str = "level.fog_vol";

  /// The only version the renderer loads, and the only one shipped.
  pub const CURRENT_VERSION: u16 = 3;

  /// Reads the fog bodies from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a fog body list this reads.
  pub fn read_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level fog volumes were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads the fog bodies from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a fog body list this reads.
  pub fn read_from_file(file: File) -> XrfResult<Self> {
    Self::read(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a fog body list this reads.
  pub fn read_from_bytes(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads off a reader positioned at the file's first byte.
  ///
  /// # Errors
  ///
  /// Returns an error when the version is one the renderer would refuse, or the bodies do not account for the file.
  pub fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u16 = reader.read_u16()?;

    if version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected level fog volume version {version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let count: u32 = reader.read_u32()?;
    let mut volumes: Vec<LevelFogVolume> =
      reader.new_bounded_vec(count.into(), LevelFogVolume::FIXED_SIZE, "fog volumes")?;

    for _ in 0..count {
      volumes.push(LevelFogVolume::read::<D>(reader)?);
    }

    reader.assert_read("Expect all data to be read from level fog volume file")?;

    Ok(Self { version, volumes })
  }

  /// Writes the fog bodies back in the layout the renderer reads.
  ///
  /// # Errors
  ///
  /// Returns an error when a count exceeds what the format's `u32` holds.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16(self.version)?;
    writer.write_u32(narrow_format_size(self.volumes.len(), "fog volumes")?)?;

    for volume in &self.volumes {
      volume.write(writer)?;
    }

    Ok(())
  }
}

impl LevelFogVolFile {
  /// Bodies the simulation flows around, across every volume.
  pub fn get_obstacles_count(&self) -> usize {
    self.volumes.iter().map(|volume| volume.obstacles.len()).sum()
  }
}
