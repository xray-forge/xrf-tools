use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// What a respawn point spawns, `ERPpointType` (`Common/LevelGameDef.h`).
pub const RPOINT_TYPES: [(u8, &str); 3] = [(0, "actor spawn"), (1, "artefact spawn"), (2, "item spawn")];

/// One respawn point of a level, from the `RPOINT_CHUNK` table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGameRPoint {
  pub position: Vector3d<f32>,
  pub rotation: Vector3d<f32>,
  pub team: u8,
  pub kind: u8,
  /// Modes this point answers to; `0xFFFF` is every one of them.
  pub game_types: u16,
  /// The spawn preset an item point names, empty on every other kind.
  pub profile: String,
}

impl LevelGameRPoint {
  /// Bytes a point occupies before its profile string.
  pub const FIXED_SIZE: u64 = 12 + 12 + 1 + 1 + 2;

  /// What this point spawns, named, or `None` for a kind the engine gives no name.
  pub fn get_kind_label(&self) -> Option<&'static str> {
    RPOINT_TYPES
      .into_iter()
      .find_map(|(value, label)| (value == self.kind).then_some(label))
  }
}

impl ChunkReadWrite for LevelGameRPoint {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      rotation: reader.read_xr::<T, _>()?,
      team: reader.read_u8()?,
      kind: reader.read_u8()?,
      game_types: reader.read_u16::<T>()?,
      profile: reader.read_w1251_string()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.rotation)?;
    writer.write_u8(self.team)?;
    writer.write_u8(self.kind)?;
    writer.write_u16::<T>(self.game_types)?;
    writer.write_w1251_string(&self.profile)?;

    Ok(())
  }
}
