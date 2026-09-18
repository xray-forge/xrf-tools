use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::constants::NET_ACTION_SPAWN;

/// One object a level spawns, as `CSE_Abstract::Spawn_Write` frames it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObject {
  /// The chunk the object was found under. Every shipped file numbers them from zero, and the engine iterates
  /// positionally rather than looking one up, so the value is carried rather than relied on.
  pub id: u32,
  /// `M_SPAWN`, which the engine asserts on before reading anything else.
  pub net_action: u16,
  /// The config section the object is built from, which is what it is.
  pub section: String,
  /// The name the level gives this one, `s_name_replace`.
  pub name: String,
  pub script_game_id: u8,
  pub script_rp: u8,
  pub position: Vector3d,
  pub direction: Vector3d,
  /// The rest of the packet, which is version- and class-dependent and is not read here.
  pub payload: Vec<u8>,
}

impl LevelSpawnObject {
  /// Reads one object out of the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not open with a spawn packet.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(id: u32, reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let net_action: u16 = reader.read_u16::<T>()?;

    if net_action != NET_ACTION_SPAWN {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected net action {net_action} in level spawn chunk {id}, expected a spawn packet"
      )));
    }

    Ok(Self {
      id,
      net_action,
      section: reader.read_w1251_string()?,
      name: reader.read_w1251_string()?,
      script_game_id: reader.read_u8()?,
      script_rp: reader.read_u8()?,
      position: reader.read_xr::<T, _>()?,
      direction: reader.read_xr::<T, _>()?,
      payload: reader.read_remaining()?,
    })
  }

  /// Writes the object back as the packet it was read from.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.net_action)?;
    writer.write_w1251_string(&self.section)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_u8(self.script_game_id)?;
    writer.write_u8(self.script_rp)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.direction)?;
    writer.buffer.extend_from_slice(&self.payload);

    Ok(())
  }
}
