use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The settings the level editor generated an AI map with, `SAIParams` (`ESceneAIMapTools_Export.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAimapParams {
  /// The side of one node's square cell, in metres; the editor's default is 0.7.
  pub patch_size: f32,
  /// The height of the centre of the tester the editor walks over the scene.
  pub test_height: f32,
  /// How far up a neighbour may lie and still be reached.
  pub can_up: f32,
  /// How far down a neighbour may lie and still be reached.
  pub can_down: f32,
}

impl BuildAimapParams {
  /// Bytes the settings occupy.
  pub const SERIALIZED_SIZE: u64 = 16;
}

impl ChunkReadWrite for BuildAimapParams {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      patch_size: reader.read_f32::<T>()?,
      test_height: reader.read_f32::<T>()?,
      can_up: reader.read_f32::<T>()?,
      can_down: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.patch_size)?;
    writer.write_f32::<T>(self.test_height)?;
    writer.write_f32::<T>(self.can_up)?;
    writer.write_f32::<T>(self.can_down)?;

    Ok(())
  }
}
