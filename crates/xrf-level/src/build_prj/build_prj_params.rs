use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The compiler settings the level editor builds with, `b_params` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjParams {
  /// The angle, in degrees, under which neighbouring normals are smoothed together.
  pub smooth_angle: f32,
  /// The distance under which vertices are welded.
  pub weld_distance: f32,
  pub lightmap_pixels_per_meter: f32,
  pub lightmap_jitter_samples: u32,
  /// The error under which a lightmap is shrunk to nothing.
  pub lightmap_rms_zero: u32,
  /// The error under which a lightmap is shrunk and recalculated.
  pub lightmap_rms: u32,
  /// `ebqDraft`, `ebqHigh` or `ebqCustom`.
  pub quality: u16,
  pub quality_reserved: u16,
  /// Six floats nothing reads, which the editor leaves holding whatever its memory held.
  pub reserved: [f32; 6],
}

impl BuildPrjParams {
  /// Bytes the settings occupy.
  pub const SERIALIZED_SIZE: u64 = 3 * 4 + 3 * 4 + 2 * 2 + 6 * 4;
}

impl ChunkReadWrite for BuildPrjParams {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let smooth_angle: f32 = reader.read_f32::<T>()?;
    let weld_distance: f32 = reader.read_f32::<T>()?;
    let lightmap_pixels_per_meter: f32 = reader.read_f32::<T>()?;
    let lightmap_jitter_samples: u32 = reader.read_u32::<T>()?;
    let lightmap_rms_zero: u32 = reader.read_u32::<T>()?;
    let lightmap_rms: u32 = reader.read_u32::<T>()?;
    let quality: u16 = reader.read_u16::<T>()?;
    let quality_reserved: u16 = reader.read_u16::<T>()?;
    let mut reserved: [f32; 6] = [0.0; 6];

    for value in &mut reserved {
      *value = reader.read_f32::<T>()?;
    }

    Ok(Self {
      smooth_angle,
      weld_distance,
      lightmap_pixels_per_meter,
      lightmap_jitter_samples,
      lightmap_rms_zero,
      lightmap_rms,
      quality,
      quality_reserved,
      reserved,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.smooth_angle)?;
    writer.write_f32::<T>(self.weld_distance)?;
    writer.write_f32::<T>(self.lightmap_pixels_per_meter)?;
    writer.write_u32::<T>(self.lightmap_jitter_samples)?;
    writer.write_u32::<T>(self.lightmap_rms_zero)?;
    writer.write_u32::<T>(self.lightmap_rms)?;
    writer.write_u16::<T>(self.quality)?;
    writer.write_u16::<T>(self.quality_reserved)?;

    for value in self.reserved {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
