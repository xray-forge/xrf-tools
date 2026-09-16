use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// Which of a modifier's values the engine mixes in, `EEnvModUsedParams` (`Common/LevelGameDef.h`).
pub const ENV_MOD_PARAMETERS: [(u16, &str); 6] = [
  (1 << 0, "view distance"),
  (1 << 1, "fog colour"),
  (1 << 2, "fog density"),
  (1 << 3, "ambient colour"),
  (1 << 4, "sky colour"),
  (1 << 5, "hemi colour"),
];

/// One local override of the level's weather, `CEnvModifier` (`xrEngine/Environment_misc.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvModifier {
  pub position: Vector3d<f32>,
  pub radius: f32,
  pub power: f32,
  pub far_plane: f32,
  pub fog_color: Vector3d<f32>,
  pub fog_density: f32,
  pub ambient: Vector3d<f32>,
  pub sky_color: Vector3d<f32>,
  pub hemi_color: Vector3d<f32>,
  /// Which values the engine actually mixes in. A file below version `0x0016` carries none and the engine uses all
  /// of them, which is what `use_flags.one()` does before the read.
  pub use_flags: Option<u16>,
}

impl EnvModifier {
  /// Bytes a modifier occupies before the flag word version `0x0016` appended.
  pub const SERIALIZED_SIZE: u64 = 12 + 4 + 4 + 4 + 12 + 4 + 12 + 12 + 12;

  /// The version that appended the flag word.
  pub const FLAGS_VERSION: u32 = 0x0016;

  /// Reads one modifier, `CEnvModifier::load`.
  ///
  /// Takes the version rather than reading one, because the file holds it once ahead of every record.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the record.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, version: u32) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      radius: reader.read_f32::<T>()?,
      power: reader.read_f32::<T>()?,
      far_plane: reader.read_f32::<T>()?,
      fog_color: reader.read_xr::<T, _>()?,
      fog_density: reader.read_f32::<T>()?,
      ambient: reader.read_xr::<T, _>()?,
      sky_color: reader.read_xr::<T, _>()?,
      hemi_color: reader.read_xr::<T, _>()?,
      use_flags: if version >= Self::FLAGS_VERSION {
        Some(reader.read_u16::<T>()?)
      } else {
        None
      },
    })
  }

  /// Writes one modifier back.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_f32::<T>(self.radius)?;
    writer.write_f32::<T>(self.power)?;
    writer.write_f32::<T>(self.far_plane)?;
    writer.write_xr::<T, _>(&self.fog_color)?;
    writer.write_f32::<T>(self.fog_density)?;
    writer.write_xr::<T, _>(&self.ambient)?;
    writer.write_xr::<T, _>(&self.sky_color)?;
    writer.write_xr::<T, _>(&self.hemi_color)?;

    if let Some(flags) = self.use_flags {
      writer.write_u16::<T>(flags)?;
    }

    Ok(())
  }

  /// The values this modifier mixes in, named. A file carrying no flag word mixes in all of them.
  pub fn get_used_parameters(&self) -> Vec<&'static str> {
    let flags: u16 = self.use_flags.unwrap_or(u16::MAX);

    ENV_MOD_PARAMETERS
      .into_iter()
      .filter_map(|(mask, name)| (flags & mask != 0).then_some(name))
      .collect()
  }
}
