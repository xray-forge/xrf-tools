use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;
use crate::level::level_snd_static_window::SndStaticWindow;

/// One sound the level plants, `SStaticSound` (`xrGame/level_sounds.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SndStaticSound {
  /// The sound it plays, which names a file.
  pub sound: String,
  pub position: Vector3d<f32>,
  pub volume: f32,
  pub frequency: f32,
  /// Hours of the day the sound is allowed to play at all.
  pub active: SndStaticWindow,
  /// How long one playing lasts, in milliseconds.
  pub play: SndStaticWindow,
  /// How long the sound waits between playings, in milliseconds.
  pub pause: SndStaticWindow,
}

impl SndStaticSound {
  /// The one chunk a record's payload sits in, which the engine finds inside the record's own chunk.
  pub const DATA_CHUNK_ID: u32 = 0;
}

impl ChunkReadWrite for SndStaticSound {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      sound: reader.read_w1251_string()?,
      position: reader.read_xr::<T, _>()?,
      volume: reader.read_f32::<T>()?,
      frequency: reader.read_f32::<T>()?,
      active: reader.read_xr::<T, _>()?,
      play: reader.read_xr::<T, _>()?,
      pause: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.sound)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.volume)?;
    writer.write_f32::<T>(self.frequency)?;
    writer.write_xr::<T, _>(&self.active)?;
    writer.write_xr::<T, _>(&self.play)?;
    writer.write_xr::<T, _>(&self.pause)?;

    Ok(())
  }
}
