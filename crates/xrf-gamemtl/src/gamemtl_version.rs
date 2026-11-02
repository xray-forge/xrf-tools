use std::fmt::{Display, Formatter};

use serde::Serialize;

/// Which game the engine takes a library for, `EGameMtlVersion` (`xrMaterialSystem/GameMtlLib.h`).
///
/// Every game wrote version 1, so the engine tells them apart by the chunks the materials carry: density came with Clear
/// Sky and the multiplayer shoot factor with Call of Pripyat. Ordered as the engine compares them, and the game reads
/// ammunition, artefacts, bone protection and stalker fire differently by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GameMtlVersion {
  /// `GAMEMTL_VERSION_SOC`.
  ShadowOfChernobyl,
  /// `GAMEMTL_VERSION_CS`.
  ClearSky,
  /// `GAMEMTL_VERSION_COP`.
  CallOfPripyat,
}

impl Display for GameMtlVersion {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(match self {
      Self::ShadowOfChernobyl => "Shadow of Chernobyl",
      Self::ClearSky => "Clear Sky",
      Self::CallOfPripyat => "Call of Pripyat",
    })
  }
}
