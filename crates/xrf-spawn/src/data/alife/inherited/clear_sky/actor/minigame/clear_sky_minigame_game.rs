use serde::{Deserialize, Serialize};

use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_crow_killer::ClearSkyCrowKiller;
use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting::ClearSkyShooting;

/// The game a Clear Sky minigame runs, by the profile name the minigames manager saves before its state: the two
/// `CMinigames:load` builds. Any other profile leaves the game without one, so it is refused.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClearSkyMinigameGame {
  CrowKiller(ClearSkyCrowKiller),
  Shooting(ClearSkyShooting),
}

impl ClearSkyMinigameGame {
  /// The profile name the minigames manager saves for the game.
  pub fn get_profile(&self) -> &'static str {
    match self {
      Self::CrowKiller(_) => ClearSkyCrowKiller::PROFILE,
      Self::Shooting(_) => ClearSkyShooting::PROFILE,
    }
  }
}
