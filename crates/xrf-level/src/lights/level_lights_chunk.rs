use std::io::Write;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::lights::level_light::LevelLight;

/// One chunk of a compiled light list, which is a run of lights or something this does not read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum LevelLightsChunk {
  /// A run of compiled lights, under the id the file stores it at.
  Lights { id: u32, lights: Vec<LevelLight> },
  /// A payload that is not whole lights, kept exactly as the file holds it.
  Opaque { id: u32, data: Vec<u8> },
}

impl LevelLightsChunk {
  /// `fsL_HEADER`, which is the one chunk `CLight_DB::LoadHemi` opens.
  pub const HEMI_CHUNK_ID: u32 = 1;

  /// The id this chunk was found under, written back unchanged.
  pub const fn get_id(&self) -> u32 {
    match self {
      Self::Lights { id, .. } | Self::Opaque { id, .. } => *id,
    }
  }

  /// The lights this chunk holds, which is none for a payload that is not a run of them.
  pub fn get_lights(&self) -> &[LevelLight] {
    match self {
      Self::Lights { lights, .. } => lights,
      Self::Opaque { .. } => &[],
    }
  }

  /// Reads one chunk, taking it as lights only where its payload divides into whole records.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload cannot be read.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(id: u32, reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let size: u64 = reader.read_bytes_remain();

    if size == 0 || !size.is_multiple_of(LevelLight::SERIALIZED_SIZE) {
      return Ok(Self::Opaque {
        id,
        data: reader.read_remaining()?,
      });
    }

    let count: u64 = size / LevelLight::SERIALIZED_SIZE;
    let mut lights: Vec<LevelLight> = reader.new_bounded_vec(count, LevelLight::SERIALIZED_SIZE, "level lights")?;

    for _ in 0..count {
      lights.push(reader.read_xr::<T, _>()?);
    }

    reader.assert_read("Expect all data to be read from level lights chunk")?;

    Ok(Self::Lights { id, lights })
  }

  /// Writes the chunk's payload back.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Lights { lights, .. } => {
        for light in lights {
          writer.write_xr::<T, _>(light)?;
        }
      }
      Self::Opaque { data, .. } => {
        writer.write_all(data)?;
      }
    }

    Ok(())
  }
}
