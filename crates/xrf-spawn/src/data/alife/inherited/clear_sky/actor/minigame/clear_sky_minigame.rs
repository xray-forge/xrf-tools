use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_crow_killer::ClearSkyCrowKiller;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_minigame_game::ClearSkyMinigameGame;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting::ClearSkyShooting;

/// One minigame of Clear Sky's minigames manager, as `CMinigames:save` writes it
/// (`gamedata-cs/scripts/minigames_manager.script`): its name, its game's profile name, its state, then the game's own
/// state.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSkyMinigame {
  pub name: String,
  /// What the manager holds it as: `valiable` (sic), `enabled`, `started` or `running`.
  pub state: String,
  pub game: ClearSkyMinigameGame,
}

impl ClearSkyMinigame {
  /// The state `CMinigames`' constructor gives each minigame, spelled as the script spells it.
  pub(crate) const CONSTRUCTED_STATE: &'static str = "valiable";

  /// Import the minigame kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    let profile: String = section.read_field(&format!("{prefix}.profile"))?;

    Ok(Self {
      name: section.read_field(&format!("{prefix}.name"))?,
      state: section.read_field(&format!("{prefix}.state"))?,
      game: match profile.as_str() {
        ClearSkyCrowKiller::PROFILE => ClearSkyMinigameGame::CrowKiller(ClearSkyCrowKiller::import(section, prefix)?),
        ClearSkyShooting::PROFILE => ClearSkyMinigameGame::Shooting(ClearSkyShooting::import(section, prefix)?),
        other => return Err(Self::new_unknown_profile_error(other)),
      },
    })
  }

  /// Export the minigame under `prefix`.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    ltx
      .with_section(section_name)
      .set(format!("{prefix}.name"), &self.name)
      .set(format!("{prefix}.profile"), self.game.get_profile())
      .set(format!("{prefix}.state"), &self.state);

    match &self.game {
      ClearSkyMinigameGame::CrowKiller(game) => game.export(section_name, prefix, ltx),
      ClearSkyMinigameGame::Shooting(game) => game.export(section_name, prefix, ltx),
    }
  }

  fn new_unknown_profile_error(profile: &str) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Minigame profile '{profile}' is neither {} nor {}, the games Clear Sky's minigames manager loads",
      ClearSkyCrowKiller::PROFILE,
      ClearSkyShooting::PROFILE
    ))
  }
}

impl ChunkFieldReadWrite for ClearSkyMinigame {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let profile: String = reader.read_w1251_string()?;
    let state: String = reader.read_w1251_string()?;

    Ok(Self {
      name,
      state,
      game: match profile.as_str() {
        ClearSkyCrowKiller::PROFILE => ClearSkyMinigameGame::CrowKiller(reader.read_xr::<T, _>()?),
        ClearSkyShooting::PROFILE => ClearSkyMinigameGame::Shooting(reader.read_xr::<T, _>()?),
        other => return Err(Self::new_unknown_profile_error(other)),
      },
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_w1251_string(self.game.get_profile())?;
    writer.write_w1251_string(&self.state)?;

    match &self.game {
      ClearSkyMinigameGame::CrowKiller(game) => writer.write_xr::<T, _>(game),
      ClearSkyMinigameGame::Shooting(game) => writer.write_xr::<T, _>(game),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_minigame::ClearSkyMinigame;

  #[test]
  fn refuses_a_profile_the_manager_does_not_load() -> XrfResult {
    let error: String = ChunkReader::<InMemoryChunkDataSource>::from_bytes(b"test\0CMGDarts\0valiable\0")?
      .read_xr::<XRayByteOrder, ClearSkyMinigame>()
      .expect_err("an unknown profile")
      .to_string();

    assert!(error.contains("Minigame profile 'CMGDarts'"), "{error}");

    Ok(())
  }
}
