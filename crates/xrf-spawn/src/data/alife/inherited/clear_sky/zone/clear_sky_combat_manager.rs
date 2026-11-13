use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::constants::NIL;
use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;
use crate::data::generic::script_to_string::ScriptToString;

/// The combat state a Clear Sky smart terrain saves, `CCombat_manager:save` with its `CCover_manager:save`
/// (`gamedata-cs/scripts/sim_combat.script`, `cover_manager.script`), each closed by its own save marker.
///
/// The squads in combat, the squads forced online, the cover table and the time the actor was last seen are records
/// only a running simulation fills; a spawn holds none of them, so their counts and flags are read as empty and any
/// other value is refused rather than read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSkyCombatManager {
  /// `actor_defence_come`, a `w_bool`.
  pub actor_defence_come: u8,
  /// `combat_quest` as written; the constructor's is the text `nil`.
  pub combat_quest: String,
  /// The task's id, `u16::MAX` for none.
  pub task_id: u16,
  /// `see_actor_enemy` as the script writes it, `tostring` of a value; `None` for `nil`.
  pub see_actor_enemy: Option<String>,
  /// `force_online`, a `w_bool`.
  pub force_online: u8,
  /// The cover manager's `is_valid`, a `w_bool`.
  pub cover_is_valid: u8,
}

impl ClearSkyCombatManager {
  const SCRIPT_CLASS: &'static str = "CCombat_manager";
  const COVER_SCRIPT_CLASS: &'static str = "CCover_manager";

  /// Import the state kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    Ok(Self {
      actor_defence_come: section.read_field(&format!("{prefix}.actor_defence_come"))?,
      combat_quest: section.read_field(&format!("{prefix}.combat_quest"))?,
      task_id: section.read_field(&format!("{prefix}.task_id"))?,
      see_actor_enemy: ScriptToString::parse_text(section.read_field(&format!("{prefix}.see_actor_enemy"))?),
      force_online: section.read_field(&format!("{prefix}.force_online"))?,
      cover_is_valid: section.read_field(&format!("{prefix}.cover_is_valid"))?,
    })
  }

  /// Export the state under `prefix`.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    ltx
      .with_section(section_name)
      .set(format!("{prefix}.actor_defence_come"), self.actor_defence_come)
      .set(format!("{prefix}.combat_quest"), &self.combat_quest)
      .set(format!("{prefix}.task_id"), self.task_id)
      .set(
        format!("{prefix}.see_actor_enemy"),
        ScriptToString::format_text(self.see_actor_enemy.as_deref()),
      )
      .set(format!("{prefix}.force_online"), self.force_online)
      .set(format!("{prefix}.cover_is_valid"), self.cover_is_valid);
  }
}

/// What `CCombat_manager`'s constructor holds, which the Level Editor's form takes in place of saved fields.
impl Default for ClearSkyCombatManager {
  fn default() -> Self {
    Self {
      actor_defence_come: 0,
      combat_quest: String::from(NIL),
      task_id: u16::MAX,
      see_actor_enemy: None,
      force_online: 0,
      cover_is_valid: 0,
    }
  }
}

impl ChunkFieldReadWrite for ClearSkyCombatManager {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    ScriptSaveMarker::read::<T, _, _>(reader, Self::SCRIPT_CLASS, |reader| {
      let actor_defence_come: u8 = reader.read_u8()?;
      let combat_quest: String = reader.read_w1251_string()?;
      let task_id: u16 = reader.read_u16::<T>()?;
      let see_actor_enemy: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);

      ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "time the actor was seen flag", 0)?;
      ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "squads count", 0)?;

      let force_online: u8 = reader.read_u8()?;

      ScriptAbsentState::read_u8(reader, Self::SCRIPT_CLASS, "squads forced online count", 0)?;

      let cover_is_valid: u8 = ScriptSaveMarker::read::<T, _, _>(reader, Self::COVER_SCRIPT_CLASS, |reader| {
        let is_valid: u8 = reader.read_u8()?;

        ScriptAbsentState::read_u8(reader, Self::COVER_SCRIPT_CLASS, "covers count", 0)?;

        Ok(is_valid)
      })?;

      Ok(Self {
        actor_defence_come,
        combat_quest,
        task_id,
        see_actor_enemy,
        force_online,
        cover_is_valid,
      })
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    ScriptSaveMarker::write::<T, _>(writer, Self::SCRIPT_CLASS, |writer| {
      writer.write_u8(self.actor_defence_come)?;
      writer.write_w1251_string(&self.combat_quest)?;
      writer.write_u16::<T>(self.task_id)?;
      writer.write_w1251_string(ScriptToString::format_text(self.see_actor_enemy.as_deref()))?;
      // No time the actor was seen, and no squads in combat.
      writer.write_u8(0)?;
      writer.write_u8(0)?;
      writer.write_u8(self.force_online)?;
      // No squads forced online.
      writer.write_u8(0)?;

      ScriptSaveMarker::write::<T, _>(writer, Self::COVER_SCRIPT_CLASS, |writer| {
        writer.write_u8(self.cover_is_valid)?;
        // No covers.
        writer.write_u8(0)
      })
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::inherited::clear_sky::zone::clear_sky_combat_manager::ClearSkyCombatManager;

  /// The constructor's state, as every Clear Sky level's smart terrain saves it: the cover manager's two bytes and its
  /// marker sit inside the combat manager's, which counts them.
  const CONSTRUCTED: &[u8] = b"\0nil\0\xff\xffnil\0\0\0\0\0\0\0\x02\0\x13\0";

  #[test]
  fn reads_and_writes_the_constructed_state() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ClearSkyCombatManager::default().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, CONSTRUCTED);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(CONSTRUCTED)?;

    assert_eq!(
      ClearSkyCombatManager::read::<XRayByteOrder, _>(&mut reader)?,
      ClearSkyCombatManager::default()
    );
    assert!(reader.is_ended());

    Ok(())
  }

  /// A squad in combat would be followed by its record, which is not read.
  #[test]
  fn refuses_squads_in_combat() -> XrfResult {
    let mut bytes: Vec<u8> = CONSTRUCTED.to_vec();

    // Past the defence flag, the quest, the task and the enemy, then the seen time's flag.
    bytes[12] = 1;

    let error: String = ClearSkyCombatManager::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&bytes)?)
      .expect_err("one squad in combat")
      .to_string();

    assert!(error.contains("CCombat_manager holds squads count 1"), "{error}");

    Ok(())
  }

  #[test]
  fn imports_and_exports_a_state() -> XrfResult {
    let original: ClearSkyCombatManager = ClearSkyCombatManager {
      actor_defence_come: 1,
      combat_quest: String::from("defend"),
      task_id: 12,
      see_actor_enemy: Some(String::from("true")),
      force_online: 1,
      cover_is_valid: 1,
    };
    let mut ltx: Ltx = Ltx::new();

    original.export("object", "smart_terrain.combat", &mut ltx);

    assert_eq!(
      ClearSkyCombatManager::import(ltx.section("object").expect("the section"), "smart_terrain.combat")?,
      original
    );

    Ok(())
  }
}
