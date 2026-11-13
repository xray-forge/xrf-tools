use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_utils::narrow_format_size;

use crate::data::generic::script_save_marker::ScriptSaveMarker;

/// The crow shooting minigame's state, `CMGCrowKiller:save` (`gamedata-cs/scripts/minigame_crowkiller.script`): the
/// parameters the minigames manager keeps for it, then the game's own counters, closed by a save marker.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSkyCrowKiller {
  pub highscore: u8,
  pub timer: u16,
  /// The parameter `win`, a `w_bool`.
  pub win: u8,
  pub crows_to_kill: Vec<u8>,
  pub money_multiplier: u16,
  pub champion_multiplier: u16,
  pub selected: u8,
  pub game_type: String,
  /// The game's own `high_score`.
  pub current_high_score: u8,
  /// The game's own `timer`.
  pub current_timer: u16,
  pub time_out: i16,
  pub killed_counter: u8,
  /// The game's own `win`, a `w_bool`.
  pub current_win: u8,
}

impl ClearSkyCrowKiller {
  /// The profile name the minigames manager saves for it.
  pub(crate) const PROFILE: &'static str = "CMGCrowKiller";

  /// Import the state kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    let key = |name: &str| format!("{prefix}.{name}");

    Ok(Self {
      highscore: section.read_field(&key("highscore"))?,
      timer: section.read_field(&key("timer"))?,
      win: section.read_field(&key("win"))?,
      crows_to_kill: Self::parse_crows(&section.read_field::<String>(&key("crows_to_kill"))?)?,
      money_multiplier: section.read_field(&key("money_multiplier"))?,
      champion_multiplier: section.read_field(&key("champion_multiplier"))?,
      selected: section.read_field(&key("selected"))?,
      game_type: section.read_field(&key("game_type"))?,
      current_high_score: section.read_field(&key("current_high_score"))?,
      current_timer: section.read_field(&key("current_timer"))?,
      time_out: section.read_field(&key("time_out"))?,
      killed_counter: section.read_field(&key("killed_counter"))?,
      current_win: section.read_field(&key("current_win"))?,
    })
  }

  /// Export the state under `prefix`, the crows to kill as a comma list.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    let key = |name: &str| format!("{prefix}.{name}");
    let crows: Vec<String> = self.crows_to_kill.iter().map(u8::to_string).collect();

    ltx
      .with_section(section_name)
      .set(key("highscore"), self.highscore)
      .set(key("timer"), self.timer)
      .set(key("win"), self.win)
      .set(key("crows_to_kill"), crows.join(","))
      .set(key("money_multiplier"), self.money_multiplier)
      .set(key("champion_multiplier"), self.champion_multiplier)
      .set(key("selected"), self.selected)
      .set(key("game_type"), &self.game_type)
      .set(key("current_high_score"), self.current_high_score)
      .set(key("current_timer"), self.current_timer)
      .set(key("time_out"), self.time_out)
      .set(key("killed_counter"), self.killed_counter)
      .set(key("current_win"), self.current_win);
  }

  /// The crows to kill from their comma list, none for an empty one.
  fn parse_crows(value: &str) -> XrfResult<Vec<u8>> {
    value
      .split(',')
      .map(str::trim)
      .filter(|crows| !crows.is_empty())
      .map(|crows| {
        crows
          .parse::<u8>()
          .map_err(|_| XrfError::new_parsing_error(format!("Crows to kill '{crows}' is not a byte")))
      })
      .collect()
  }
}

/// What `CMinigames`' constructor gives the game, which a new game's actor saves.
impl Default for ClearSkyCrowKiller {
  fn default() -> Self {
    Self {
      highscore: 0,
      timer: 60,
      win: 0,
      crows_to_kill: vec![0; 4],
      money_multiplier: 10,
      champion_multiplier: 100,
      selected: 0,
      game_type: String::new(),
      current_high_score: 0,
      current_timer: 0,
      time_out: 10,
      killed_counter: 0,
      current_win: 0,
    }
  }
}

impl ChunkFieldReadWrite for ClearSkyCrowKiller {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    ScriptSaveMarker::read::<T, _, _>(reader, Self::PROFILE, |reader| {
      let highscore: u8 = reader.read_u8()?;
      let timer: u16 = reader.read_u16::<T>()?;
      let win: u8 = reader.read_u8()?;
      let crows_count: u8 = reader.read_u8()?;
      let crows_to_kill: Vec<u8> = (0..crows_count).map(|_| reader.read_u8()).collect::<XrfResult<_>>()?;

      Ok(Self {
        highscore,
        timer,
        win,
        crows_to_kill,
        money_multiplier: reader.read_u16::<T>()?,
        champion_multiplier: reader.read_u16::<T>()?,
        selected: reader.read_u8()?,
        game_type: reader.read_w1251_string()?,
        current_high_score: reader.read_u8()?,
        current_timer: reader.read_u16::<T>()?,
        // `w_s16`, the same two bytes read as signed.
        time_out: i16::from_ne_bytes(reader.read_u16::<T>()?.to_ne_bytes()),
        killed_counter: reader.read_u8()?,
        current_win: reader.read_u8()?,
      })
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    ScriptSaveMarker::write::<T, _>(writer, Self::PROFILE, |writer| {
      writer.write_u8(self.highscore)?;
      writer.write_u16::<T>(self.timer)?;
      writer.write_u8(self.win)?;
      writer.write_u8(narrow_format_size(self.crows_to_kill.len(), "crows to kill")?)?;

      for crows in &self.crows_to_kill {
        writer.write_u8(*crows)?;
      }

      writer.write_u16::<T>(self.money_multiplier)?;
      writer.write_u16::<T>(self.champion_multiplier)?;
      writer.write_u8(self.selected)?;
      writer.write_w1251_string(&self.game_type)?;
      writer.write_u8(self.current_high_score)?;
      writer.write_u16::<T>(self.current_timer)?;
      writer.write_u16::<T>(u16::from_ne_bytes(self.time_out.to_ne_bytes()))?;
      writer.write_u8(self.killed_counter)?;
      writer.write_u8(self.current_win)
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_crow_killer::ClearSkyCrowKiller;

  /// The constructor's state as a Clear Sky level's actor saves it, closed by its marker counting 22 bytes.
  const CONSTRUCTED: &[u8] = b"\0\x3c\0\0\x04\0\0\0\0\x0a\0\x64\0\0\0\0\0\0\x0a\0\0\0\x16\0";

  #[test]
  fn reads_and_writes_the_constructed_state() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ClearSkyCrowKiller::default().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, CONSTRUCTED);
    assert_eq!(
      ClearSkyCrowKiller::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(
        CONSTRUCTED
      )?)?,
      ClearSkyCrowKiller::default()
    );

    Ok(())
  }

  #[test]
  fn imports_and_exports_a_state() -> XrfResult {
    for crows_to_kill in [vec![], vec![3, 5, 7, 9]] {
      let original: ClearSkyCrowKiller = ClearSkyCrowKiller {
        crows_to_kill,
        game_type: String::from("crows"),
        time_out: -1,
        ..ClearSkyCrowKiller::default()
      };
      let mut ltx: Ltx = Ltx::new();

      original.export("object", "actor.minigame.0", &mut ltx);

      assert_eq!(
        ClearSkyCrowKiller::import(ltx.section("object").expect("the section"), "actor.minigame.0")?,
        original
      );
    }

    Ok(())
  }
}
