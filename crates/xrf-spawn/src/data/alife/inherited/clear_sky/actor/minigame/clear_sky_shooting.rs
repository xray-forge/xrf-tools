use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::constants::NIL;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting_win::ClearSkyShootingWin;
use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;
use crate::data::generic::script_to_string::ScriptToString;

/// The shooting range minigame's state, `CMGShooting:save` (`gamedata-cs/scripts/minigame_shooting.script`): the
/// parameters the minigames manager keeps for it, the inventory it took from the actor and the game in progress, closed
/// by a save marker.
///
/// The targets, the inventory taken and the game in progress are what playing fills; a spawn holds no targets, nothing
/// taken and the game `nil`, so those are read as such and any other value is refused rather than read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSkyShooting {
  pub game_type: String,
  pub wpn_type: String,
  /// The patrol paths, each `tostring` of a name; `None` for `nil`.
  pub stand_way: Option<String>,
  pub look_way: Option<String>,
  pub stand_way_back: Option<String>,
  pub look_way_back: Option<String>,
  /// The target object's name; `None` for `nil`.
  pub obj_name: Option<String>,
  pub win: ClearSkyShootingWin,
  pub distance: u8,
  pub ammo: u8,
  pub target_counter: u8,
  pub prev_time: u32,
}

impl ClearSkyShooting {
  /// The profile name the minigames manager saves for it.
  pub(crate) const PROFILE: &'static str = "CMGShooting";

  /// Import the state kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    let key = |name: &str| format!("{prefix}.{name}");
    let text =
      |name: &str| -> XrfResult<Option<String>> { Ok(ScriptToString::parse_text(section.read_field(&key(name))?)) };

    Ok(Self {
      game_type: section.read_field(&key("game_type"))?,
      wpn_type: section.read_field(&key("wpn_type"))?,
      stand_way: text("stand_way")?,
      look_way: text("look_way")?,
      stand_way_back: text("stand_way_back")?,
      look_way_back: text("look_way_back")?,
      obj_name: text("obj_name")?,
      win: ClearSkyShootingWin::import(section, prefix)?,
      distance: section.read_field(&key("distance"))?,
      ammo: section.read_field(&key("ammo"))?,
      target_counter: section.read_field(&key("target_counter"))?,
      prev_time: section.read_field(&key("prev_time"))?,
    })
  }

  /// Export the state under `prefix`.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    let key = |name: &str| format!("{prefix}.{name}");

    ltx
      .with_section(section_name)
      .set(key("game_type"), &self.game_type)
      .set(key("wpn_type"), &self.wpn_type)
      .set(key("stand_way"), ScriptToString::format_text(self.stand_way.as_deref()))
      .set(key("look_way"), ScriptToString::format_text(self.look_way.as_deref()))
      .set(
        key("stand_way_back"),
        ScriptToString::format_text(self.stand_way_back.as_deref()),
      )
      .set(
        key("look_way_back"),
        ScriptToString::format_text(self.look_way_back.as_deref()),
      )
      .set(key("obj_name"), ScriptToString::format_text(self.obj_name.as_deref()))
      .set(key("distance"), self.distance)
      .set(key("ammo"), self.ammo)
      .set(key("target_counter"), self.target_counter)
      .set(key("prev_time"), self.prev_time);

    self.win.export(section_name, prefix, ltx);
  }
}

/// What `CMinigames`' constructor gives the game, which a new game's actor saves.
impl Default for ClearSkyShooting {
  fn default() -> Self {
    Self {
      game_type: String::new(),
      wpn_type: String::new(),
      stand_way: None,
      look_way: None,
      stand_way_back: None,
      look_way_back: None,
      obj_name: None,
      win: ClearSkyShootingWin::Count(0),
      distance: 0,
      ammo: 0,
      target_counter: 0,
      prev_time: 0,
    }
  }
}

impl ChunkFieldReadWrite for ClearSkyShooting {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    ScriptSaveMarker::read::<T, _, _>(reader, Self::PROFILE, |reader| {
      let game_type: String = reader.read_w1251_string()?;
      let wpn_type: String = reader.read_w1251_string()?;
      let stand_way: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);
      let look_way: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);
      let stand_way_back: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);
      let look_way_back: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);
      let obj_name: Option<String> = ScriptToString::parse_text(reader.read_w1251_string()?);
      let win: ClearSkyShootingWin = reader.read_xr::<T, _>()?;
      let distance: u8 = reader.read_u8()?;
      let ammo: u8 = reader.read_u8()?;

      ScriptAbsentState::read_u8(reader, Self::PROFILE, "targets count", 0)?;

      let target_counter: u8 = reader.read_u8()?;

      ScriptAbsentState::read_u8(reader, Self::PROFILE, "inventory items count", 0)?;

      let prev_time: u32 = reader.read_u32::<T>()?;

      ScriptAbsentState::read_string(reader, Self::PROFILE, "current game", NIL)?;

      Ok(Self {
        game_type,
        wpn_type,
        stand_way,
        look_way,
        stand_way_back,
        look_way_back,
        obj_name,
        win,
        distance,
        ammo,
        target_counter,
        prev_time,
      })
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    ScriptSaveMarker::write::<T, _>(writer, Self::PROFILE, |writer| {
      writer.write_w1251_string(&self.game_type)?;
      writer.write_w1251_string(&self.wpn_type)?;

      for text in [
        &self.stand_way,
        &self.look_way,
        &self.stand_way_back,
        &self.look_way_back,
        &self.obj_name,
      ] {
        writer.write_w1251_string(ScriptToString::format_text(text.as_deref()))?;
      }

      writer.write_xr::<T, _>(&self.win)?;
      writer.write_u8(self.distance)?;
      writer.write_u8(self.ammo)?;
      // No targets.
      writer.write_u8(0)?;
      writer.write_u8(self.target_counter)?;
      // No inventory taken.
      writer.write_u8(0)?;
      writer.write_u32::<T>(self.prev_time)?;
      // No game in progress.
      writer.write_w1251_string(NIL)
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting::ClearSkyShooting;
  use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting_win::ClearSkyShootingWin;

  /// The constructor's state as a Clear Sky level's actor saves it, closed by its marker counting 38 bytes.
  const CONSTRUCTED: &[u8] = b"\0\0nil\0nil\0nil\0nil\0nil\0\x01\0\0\0\0\0\0\0\0\0\0\0nil\0\x26\0";

  #[test]
  fn reads_and_writes_the_constructed_state() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ClearSkyShooting::default().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, CONSTRUCTED);
    assert_eq!(
      ClearSkyShooting::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(
        CONSTRUCTED
      )?)?,
      ClearSkyShooting::default()
    );

    Ok(())
  }

  /// A game in progress would be followed by its own state, which is not read.
  #[test]
  fn refuses_a_game_in_progress() -> XrfResult {
    let mut bytes: Vec<u8> = CONSTRUCTED[..CONSTRUCTED.len() - 6].to_vec();

    bytes.extend_from_slice(b"count\0");

    let error: String =
      ClearSkyShooting::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(&bytes)?)
        .expect_err("a game in progress")
        .to_string();

    assert!(error.contains("CMGShooting holds current game \"count\""), "{error}");

    Ok(())
  }

  #[test]
  fn imports_and_exports_a_state() -> XrfResult {
    let original: ClearSkyShooting = ClearSkyShooting {
      game_type: String::from("count"),
      stand_way: Some(String::from("shooting_way")),
      win: ClearSkyShootingWin::Flag(1),
      prev_time: 90_000,
      ..ClearSkyShooting::default()
    };
    let mut ltx: Ltx = Ltx::new();

    original.export("object", "actor.minigame.1", &mut ltx);

    assert_eq!(
      ClearSkyShooting::import(ltx.section("object").expect("the section"), "actor.minigame.1")?,
      original
    );

    Ok(())
  }
}
