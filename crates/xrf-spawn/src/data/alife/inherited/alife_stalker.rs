use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_human_stalker::AlifeObjectHumanStalker;
use crate::data::generic::offline_object_state::OfflineObjectState;

/// A stalker as its script class `se_stalker` saves one: vanilla's `se_stalker.script` and xrf-engine's `Stalker.ts`
/// both follow the server class's spawn fields with the offline state and whether the corpse's loot was dropped,
/// unmarked, and add nothing to the update packet.
///
/// The tail mirrors `STATE_Write` in xrf-engine's `src/engine/core/objects/creature/Stalker.ts`, whose own test pins
/// its write order, so a change to either side changes the other in the same work.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeStalker {
  pub base: AlifeObjectHumanStalker,
  pub offline_state: OfflineObjectState,
  /// Whether the death manager already dropped the corpse's loot, `w_bool` of `death_droped` (`isCorpseLootDropped`
  /// in xrf-engine); zero for a new object.
  pub loot_dropped: u8,
}

impl AlifeStalker {
  /// Place the stalker at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

#[cfg(test)]
impl AlifeStalker {
  /// A stalker at `position` as a compiled spawn holds one.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectHumanStalker::new_mock(position),
      offline_state: OfflineObjectState::default(),
      loot_dropped: 0,
    }
  }
}

impl AlifePacketsReadWrite for AlifeStalker {
  /// Read the stalker from both packets; the script class adds to the spawn packet alone.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectHumanStalker::read::<T, _>(spawn, update)?,
      offline_state: spawn.read_xr::<T, _>()?,
      loot_dropped: spawn.read_u8()?,
    })
  }

  /// Write the stalker into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_xr::<T, _>(&self.offline_state)?;
    spawn.write_u8(self.loot_dropped)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeStalker {
  /// Import stalker data from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectHumanStalker::import(section_name, ltx)?,
      offline_state: OfflineObjectState::import(section, "stalker")?,
      loot_dropped: section.read_field("stalker.loot_dropped")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.offline_state.export(section_name, "stalker", ltx);

    ltx
      .with_section(section_name)
      .set("stalker.loot_dropped", self.loot_dropped.to_string());

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
  use crate::data::alife::inherited::alife_stalker::AlifeStalker;
  use crate::data::generic::offline_object_state::OfflineObjectState;
  use crate::fixtures::GameWrittenPackets;

  /// What the game writes for a new stalker reads whole and writes back byte for byte.
  #[test]
  fn reads_and_writes_what_the_game_writes() -> XrfResult {
    let spawn: Vec<u8> = GameWrittenPackets::bytes(GameWrittenPackets::STALKER_SPAWN);
    let update: Vec<u8> = GameWrittenPackets::bytes(GameWrittenPackets::STALKER_UPDATE);
    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update)?;
    let stalker: AlifeStalker = AlifeStalker::read::<XRayByteOrder, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the game's spawn packet to be read whole")?;
    update_reader.assert_read("Expect the game's update packet to be read whole")?;

    assert_eq!(stalker.base.base.trader.character_profile, "sim_default_stalker_0");
    assert_eq!(
      stalker.base.base.base.base.base.visual_name,
      "actors\\stalker_neutral\\stalker_neutral_1"
    );
    assert_eq!(stalker.base.base.equipment_preferences.len(), 5);
    assert_eq!(stalker.base.base.main_weapon_preferences.len(), 4);
    assert_eq!(
      stalker.base.base.base.base.update_position,
      Vector3d::new(-400.0, 20.0, 700.0)
    );
    assert_eq!(stalker.base.update_start_dialog, "");
    assert_eq!(
      (&stalker.offline_state, stalker.loot_dropped),
      (&OfflineObjectState::default(), 0)
    );

    let mut spawn_writer: ChunkWriter = ChunkWriter::new();
    let mut update_writer: ChunkWriter = ChunkWriter::new();

    stalker.write::<XRayByteOrder>(&mut spawn_writer, &mut update_writer)?;

    assert_eq!((spawn_writer.buffer, update_writer.buffer), (spawn, update));

    Ok(())
  }

  /// The tail in `Stalker.ts`'s `STATE_Write` order - the level vertex and active section as strings, then the loot
  /// flag - reads as the offline state and flag it saved.
  #[test]
  fn reads_the_tail_in_the_script_order() -> XrfResult {
    let original: AlifeStalker = AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    original.base.write::<XRayByteOrder>(&mut spawn, &mut update)?;
    spawn.buffer.extend_from_slice(b"435\0test_section\0\x01");

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&update.buffer)?;
    let read: AlifeStalker = AlifeStalker::read::<XRayByteOrder, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect the tail to close the spawn packet")?;

    assert_eq!(
      (read.offline_state, read.loot_dropped),
      (
        OfflineObjectState {
          level_vertex_id: Some(435),
          active_section: Some(String::from("test_section")),
        },
        1
      )
    );

    Ok(())
  }

  /// A compiled spawn's tail: two `nil` strings and the loot byte close the spawn packet, with no marker counting them.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeStalker = AlifeStalker::new_mock(Vector3d::new(-12.5, 0.75, 310.0));
    let (base_spawn_size, base_update_size, _) = original.base.read_back::<XRayByteOrder>()?;
    let mut spawn: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut spawn, &mut ChunkWriter::new())?;

    assert_eq!(&spawn.buffer[base_spawn_size..], b"nil\0nil\0\0");
    assert_eq!(
      original.read_back::<XRayByteOrder>()?,
      (base_spawn_size + 9, base_update_size, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeStalker = AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.offline_state.active_section = Some(String::from("walker@camp"));
    original.loot_dropped = 1;
    original.export("data", &mut ltx)?;

    assert_eq!(AlifeStalker::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeStalker = AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      serde_json::from_str::<AlifeStalker>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
