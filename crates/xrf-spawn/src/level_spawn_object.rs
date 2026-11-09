use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::constants::NET_ACTION_SPAWN;
use crate::data::alife::alife_object::AlifeObject;

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

  /// The object typed as its section's class, read from its spawn packet alone
  /// ([`AlifeObject::read_spawn_packet`]).
  ///
  /// # Errors
  ///
  /// Returns an error when the packet is not of a known class, or holds bytes past what its class reads.
  pub fn read_alife_object<T: ByteOrder>(&self) -> XrfResult<AlifeObject> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    self.write::<T>(&mut bytes)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes.buffer)?;
    let object: AlifeObject = AlifeObject::read_spawn_packet::<T, _>(&mut reader)?;

    if !reader.is_ended() {
      return Err(XrfError::new_parsing_error(format!(
        "Level spawn object '{}' holds {} bytes past what its class reads",
        self.name,
        reader.read_bytes_remain()
      )));
    }

    Ok(object)
  }

  /// The object a level spawns under chunk `id` from a typed object's spawn packet.
  ///
  /// # Errors
  ///
  /// Returns an error when the object's payload is not its section's class, or the writer refuses a value.
  pub fn from_alife_object<T: ByteOrder>(id: u32, object: &AlifeObject) -> XrfResult<Self> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write_spawn_packet::<T, _>(&mut bytes)?;

    Self::read::<T, _>(id, &mut ChunkReader::from_bytes(&bytes.buffer)?)
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_object::AlifeObject;
  use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
  use crate::data::alife::inherited::alife_graph_point::AlifeGraphPoint;
  use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
  use crate::data::meta::cls_id::ClsId;
  use crate::level_spawn_object::LevelSpawnObject;

  fn graph_point() -> AlifeObject {
    AlifeObject {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from("graph_point"),
        name: String::from("test_graph_point"),
        script_game_id: 0,
        script_rp: 254,
        position: Vector3d::new(1.5, -2.0, 4.25),
        direction: Vector3d::ZERO,
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        script_flags: 33,
        version: 128,
        game_type: u16::MAX,
        script_version: 12,
        client_data: Vec::new(),
        spawn_id: u16::MAX,
      },
      clsid: ClsId::AiGraph,
      inherited: AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
        connection_point_name: String::new(),
        connection_level_name: String::new(),
        location0: 1,
        location1: 0,
        location2: 0,
        location3: 0,
      })),
    }
  }

  /// A typed object listed under a chunk reads back as the same object.
  #[test]
  fn types_a_listed_object_and_lists_it_back() -> XrfResult {
    let listed: LevelSpawnObject = LevelSpawnObject::from_alife_object::<XRayByteOrder>(3, &graph_point())?;

    assert_eq!((listed.id, listed.name.as_str()), (3, "test_graph_point"));
    assert_eq!(listed.read_alife_object::<XRayByteOrder>()?, graph_point());

    Ok(())
  }

  /// Bytes past what the class reads would be dropped by a rewrite, so the object is refused.
  #[test]
  fn refuses_bytes_past_what_the_class_reads() -> XrfResult {
    let mut listed: LevelSpawnObject = LevelSpawnObject::from_alife_object::<XRayByteOrder>(0, &graph_point())?;

    listed.payload.push(7);

    let error: String = listed
      .read_alife_object::<XRayByteOrder>()
      .expect_err("a stray byte")
      .to_string();

    assert!(error.contains("holds 1 bytes past what its class reads"), "{error}");

    Ok(())
  }
}
