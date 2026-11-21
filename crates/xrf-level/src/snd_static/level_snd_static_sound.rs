use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::snd_static::level_snd_static_window::LevelSndStaticWindow;

/// One sound the level plants, `SStaticSound` (`xrGame/level_sounds.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSndStaticSound {
  /// The sound it plays, which names a file.
  pub sound: String,
  pub position: Vector3d<f32>,
  pub volume: f32,
  pub frequency: f32,
  /// The window of the game's day the sound plays in, in milliseconds from midnight (`SStaticSound::Update`).
  pub active: LevelSndStaticWindow,
  /// How long one playing lasts, in milliseconds.
  pub play: LevelSndStaticWindow,
  /// How long the sound waits between playings, in milliseconds.
  pub pause: LevelSndStaticWindow,
}

impl LevelSndStaticSound {
  /// The one chunk a record's payload sits in, which the engine finds inside the record's own chunk.
  pub const DATA_CHUNK_ID: u32 = 0;
}

impl ChunkReadWrite for LevelSndStaticSound {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      sound: reader.read_w1251_string()?,
      position: reader.read_xr()?,
      volume: reader.read_f32()?,
      frequency: reader.read_f32()?,
      active: reader.read_xr()?,
      play: reader.read_xr()?,
      pause: reader.read_xr()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.sound)?;
    writer.write_xr(&self.position)?;
    writer.write_f32(self.volume)?;
    writer.write_f32(self.frequency)?;
    writer.write_xr(&self.active)?;
    writer.write_xr(&self.play)?;
    writer.write_xr(&self.pause)?;

    Ok(())
  }
}
