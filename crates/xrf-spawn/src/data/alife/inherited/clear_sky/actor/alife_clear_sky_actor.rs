use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
#[cfg(test)]
use xrf_math::Vector3d;
use xrf_utils::narrow_format_size;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
use crate::data::alife::inherited::clear_sky::actor::clear_sky_object_collection::ClearSkyObjectCollection;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_crow_killer::ClearSkyCrowKiller;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_minigame::ClearSkyMinigame;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_minigame_game::ClearSkyMinigameGame;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting::ClearSkyShooting;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

/// The actor as Clear Sky's `se_actor` saves it (`gamedata-cs/scripts/se_actor.script`): the server class's fields, then,
/// closed by a save marker, the random task manager, the squads' id generator and the minigames manager, each closed by
/// its own marker. It adds nothing to the update packet.
///
/// The task manager (`CRandomTask:save`, `task_manager.script`) keeps the tasks given, the rewards and the upgrade
/// tasks, which a running game fills; a spawn holds none, so their counts are read as empty and any other is refused
/// rather than read, and only its task id generator is kept.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeClearSkyActor {
  pub base: AlifeObjectActor,
  /// The task manager's id generator.
  pub task_ids: ClearSkyObjectCollection,
  /// The simulation board's id generator, which numbers squads.
  pub squad_ids: ClearSkyObjectCollection,
  pub minigames: Vec<ClearSkyMinigame>,
}

/// The script fields of [`AlifeClearSkyActor`], read and written together inside its save marker.
struct ClearSkyActorScriptState {
  task_ids: ClearSkyObjectCollection,
  squad_ids: ClearSkyObjectCollection,
  minigames: Vec<ClearSkyMinigame>,
}

impl ClearSkyActorScriptState {
  /// What the managers' constructors hold, which the Level Editor's form takes in place of saved fields: generators
  /// that gave no id and `CMinigames`' two games, in the order a Clear Sky level's actor saves them.
  fn new_constructed() -> Self {
    Self {
      task_ids: ClearSkyObjectCollection::default(),
      squad_ids: ClearSkyObjectCollection::default(),
      minigames: vec![
        ClearSkyMinigame {
          name: String::from("test_crowkiller"),
          state: String::from(ClearSkyMinigame::CONSTRUCTED_STATE),
          game: ClearSkyMinigameGame::CrowKiller(ClearSkyCrowKiller::default()),
        },
        ClearSkyMinigame {
          name: String::from("test_shooting"),
          state: String::from(ClearSkyMinigame::CONSTRUCTED_STATE),
          game: ClearSkyMinigameGame::Shooting(ClearSkyShooting::default()),
        },
      ],
    }
  }
}

impl AlifeClearSkyActor {
  /// The script class the trailing fields belong to, named in a refusal.
  const SCRIPT_CLASS: &'static str = "se_actor";
  const TASK_MANAGER: &'static str = "CRandomTask";
  const MINIGAMES_MANAGER: &'static str = "CMinigames";

  fn read_script_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<ClearSkyActorScriptState> {
    let task_ids: ClearSkyObjectCollection = ScriptSaveMarker::read::<T, _, _>(reader, Self::TASK_MANAGER, |reader| {
      ScriptAbsentState::read_u16::<T, _>(reader, Self::TASK_MANAGER, "tasks count", 0)?;

      let task_ids: ClearSkyObjectCollection = reader.read_xr::<T, _>()?;

      ScriptAbsentState::read_u8(reader, Self::TASK_MANAGER, "rewards count", 0)?;
      ScriptAbsentState::read_u16::<T, _>(reader, Self::TASK_MANAGER, "upgrade tasks count", 0)?;

      Ok(task_ids)
    })?;
    let squad_ids: ClearSkyObjectCollection = reader.read_xr::<T, _>()?;
    let minigames: Vec<ClearSkyMinigame> =
      ScriptSaveMarker::read::<T, _, _>(reader, Self::MINIGAMES_MANAGER, |reader| {
        let count: u16 = reader.read_u16::<T>()?;

        (0..count).map(|_| reader.read_xr::<T, _>()).collect()
      })?;

    Ok(ClearSkyActorScriptState {
      task_ids,
      squad_ids,
      minigames,
    })
  }

  fn write_script_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    ScriptSaveMarker::write::<T, _>(writer, Self::TASK_MANAGER, |writer| {
      // No tasks given.
      writer.write_u16::<T>(0)?;
      writer.write_xr::<T, _>(&self.task_ids)?;
      // No rewards and no upgrade tasks.
      writer.write_u8(0)?;
      writer.write_u16::<T>(0)
    })?;
    writer.write_xr::<T, _>(&self.squad_ids)?;
    ScriptSaveMarker::write::<T, _>(writer, Self::MINIGAMES_MANAGER, |writer| {
      writer.write_u16::<T>(narrow_format_size(self.minigames.len(), "minigames")?)?;

      for minigame in &self.minigames {
        writer.write_xr::<T, _>(minigame)?;
      }

      Ok(())
    })
  }
}

#[cfg(test)]
impl AlifeClearSkyActor {
  /// An actor at `position` as a Clear Sky level places one, its managers as their constructors leave them.
  pub fn new_mock(position: Vector3d) -> Self {
    let state: ClearSkyActorScriptState = ClearSkyActorScriptState::new_constructed();

    Self {
      base: AlifeObjectActor::new_mock(position),
      task_ids: state.task_ids,
      squad_ids: state.squad_ids,
      minigames: state.minigames,
    }
  }
}

impl AlifePacketsReadWrite for AlifeClearSkyActor {
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
  }

  /// Read the actor from both packets. Its `STATE_Read` returns before its own fields under the Level Editor, so the
  /// editor's form ending at the server class's fields takes the managers' constructor state.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    let base: AlifeObjectActor = AlifeObjectActor::read::<T, _, _>(spawn, update, layout)?;
    let state: ClearSkyActorScriptState = ScriptSaveMarker::read_or_constructed::<T, _, _>(
      spawn,
      Self::SCRIPT_CLASS,
      ClearSkyActorScriptState::new_constructed,
      Self::read_script_state::<T, _>,
    )?;

    Ok(Self {
      base,
      task_ids: state.task_ids,
      squad_ids: state.squad_ids,
      minigames: state.minigames,
    })
  }

  /// Write the actor into both packets.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    self.base.write::<T, _, _>(spawn, update, layout)?;

    ScriptSaveMarker::write::<T, _>(spawn, Self::SCRIPT_CLASS, |writer| {
      self.write_script_state::<T, _>(writer)
    })
  }

  /// Import actor data from ltx file section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    layout.assert_clear_sky_script_version(Self::SCRIPT_CLASS)?;

    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;
    let minigames_count: u16 = section.read_field("actor.minigames")?;

    Ok(Self {
      base: AlifeObjectActor::import(section_name, ltx, layout)?,
      task_ids: ClearSkyObjectCollection::import(section, "actor.task_ids")?,
      squad_ids: ClearSkyObjectCollection::import(section, "actor.squad_ids")?,
      minigames: (0..minigames_count)
        .map(|index| ClearSkyMinigame::import(section, &format!("actor.minigame.{index}")))
        .collect::<XrfResult<_>>()?,
    })
  }

  /// Export object data into ltx file, each minigame under its index.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx, layout)?;
    self.task_ids.export(section_name, "actor.task_ids", ltx);
    self.squad_ids.export(section_name, "actor.squad_ids", ltx);

    ltx
      .with_section(section_name)
      .set("actor.minigames", self.minigames.len());

    for (index, minigame) in self.minigames.iter().enumerate() {
      minigame.export(section_name, &format!("actor.minigame.{index}"), ltx);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::clear_sky::actor::alife_clear_sky_actor::AlifeClearSkyActor;
  use crate::data::alife::spawn_layout::SpawnLayout;

  /// The managers' markers nest inside the actor's, which counts every byte after the server class's fields.
  #[test]
  fn writes_the_managers_closed_by_the_actors_marker() -> XrfResult {
    let original: AlifeClearSkyActor = AlifeClearSkyActor::new_mock(Vector3d::new(256.24, 19.81, 550.82));
    let (base_spawn_size, base_update_size, _) = original.base.read_back::<XRayByteOrder>(SpawnLayout::CLEAR_SKY)?;
    let mut spawn: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder, _, _>(&mut spawn, &mut ChunkWriter::new(), SpawnLayout::CLEAR_SKY)?;

    assert_eq!(spawn.buffer.len() - base_spawn_size, 169 + 2);
    assert_eq!(spawn.buffer[spawn.buffer.len() - 2..], [169, 0]);
    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CLEAR_SKY)?,
      (base_spawn_size + 171, base_update_size, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeClearSkyActor = AlifeClearSkyActor::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.squad_ids.count = 2;
    original.minigames[1].state = String::from("enabled");
    original.export("data", &mut ltx, SpawnLayout::CLEAR_SKY)?;

    assert_eq!(
      AlifeClearSkyActor::import("data", &ltx, SpawnLayout::CLEAR_SKY)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeClearSkyActor = AlifeClearSkyActor::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      serde_json::from_str::<AlifeClearSkyActor>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
