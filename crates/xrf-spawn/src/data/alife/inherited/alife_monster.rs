use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_monster_base::AlifeObjectMonsterBase;
use crate::data::generic::offline_object_state::OfflineObjectState;

/// A monster as its script class `se_monster` saves one: vanilla's `se_monster.script` and xrf-engine's `Monster.ts`
/// both follow the server class's spawn fields with the offline state alone, unmarked, and add nothing to the update
/// packet.
///
/// The tail mirrors `STATE_Write` in xrf-engine's `src/engine/core/objects/creature/Monster.ts`, whose own test pins
/// its write order, so a change to either side changes the other in the same work.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeMonster {
  pub base: AlifeObjectMonsterBase,
  pub offline_state: OfflineObjectState,
}

impl AlifeMonster {
  /// Place the monster at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

#[cfg(test)]
impl AlifeMonster {
  /// A monster at `position` as a compiled spawn holds one.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectMonsterBase::new_mock(position),
      offline_state: OfflineObjectState::default(),
    }
  }
}

impl AlifePacketsReadWrite for AlifeMonster {
  /// Read the monster from both packets; the script class adds to the spawn packet alone.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectMonsterBase::read::<T, _>(spawn, update)?,
      offline_state: spawn.read_xr::<T, _>()?,
    })
  }

  /// Write the monster into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_xr::<T, _>(&self.offline_state)
  }
}

impl LtxImportExport for AlifeMonster {
  /// Import monster data from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectMonsterBase::import(section_name, ltx)?,
      offline_state: OfflineObjectState::import(section, "monster")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.offline_state.export(section_name, "monster", ltx);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_monster::AlifeMonster;
  use crate::data::generic::offline_object_state::OfflineObjectState;
  use crate::fixtures::GameWrittenPackets;

  /// What the game writes for a new monster reads whole and writes back byte for byte.
  #[test]
  fn reads_and_writes_what_the_game_writes() -> XrfResult {
    let spawn: Vec<u8> = GameWrittenPackets::bytes(GameWrittenPackets::MONSTER_SPAWN);
    let update: Vec<u8> = GameWrittenPackets::bytes(GameWrittenPackets::MONSTER_UPDATE);
    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update)?;
    let monster: AlifeMonster = AlifeMonster::read::<XRayByteOrder, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the game's spawn packet to be read whole")?;
    update_reader.assert_read("Expect the game's update packet to be read whole")?;

    assert_eq!(monster.base.base.base.base.visual_name, "monsters\\dog\\dog_white");
    assert_eq!(
      monster.base.base.base.update_position,
      Vector3d::new(-400.0, 20.0, 700.0)
    );
    assert_eq!(monster.base.skeleton.name, "$editor");
    assert_eq!(monster.base.spec_object_id, u16::MAX);
    assert_eq!(monster.offline_state, OfflineObjectState::default());

    let mut spawn_writer: ChunkWriter = ChunkWriter::new();
    let mut update_writer: ChunkWriter = ChunkWriter::new();

    monster.write::<XRayByteOrder>(&mut spawn_writer, &mut update_writer)?;

    assert_eq!((spawn_writer.buffer, update_writer.buffer), (spawn, update));

    Ok(())
  }

  /// The tail in `Monster.ts`'s `STATE_Write` order - the level vertex and active section as strings - reads as the
  /// offline state it saved.
  #[test]
  fn reads_the_tail_in_the_script_order() -> XrfResult {
    let original: AlifeMonster = AlifeMonster::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    original.base.write::<XRayByteOrder>(&mut spawn, &mut update)?;
    spawn.buffer.extend_from_slice(b"450\0scheme@test\0");

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;
    let read: AlifeMonster = AlifeMonster::read::<XRayByteOrder, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the tail to close the spawn packet")?;

    assert_eq!(
      read.offline_state,
      OfflineObjectState {
        level_vertex_id: Some(450),
        active_section: Some(String::from("scheme@test")),
      }
    );

    Ok(())
  }

  /// A compiled spawn's tail: two `nil` strings follow the server class's spawn fields with no marker counting them.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeMonster = AlifeMonster::new_mock(Vector3d::new(-12.5, 0.75, 310.0));
    let (base_spawn_size, _, _) = original.base.read_back::<XRayByteOrder>()?;
    let mut spawn: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut spawn, &mut ChunkWriter::new())?;

    assert_eq!(&spawn.buffer[base_spawn_size..], b"nil\0nil\0");
    assert_eq!(
      original.read_back::<XRayByteOrder>()?,
      (base_spawn_size + 8, 52, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeMonster = AlifeMonster::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.offline_state.level_vertex_id = Some(3108);
    original.export("data", &mut ltx)?;

    assert_eq!(AlifeMonster::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeMonster = AlifeMonster::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      serde_json::from_str::<AlifeMonster>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
