/// Which game the engine takes a library for, `EGameMtlVersion` (`xrMaterialSystem/GameMtlLib.h`).
///
/// Every game wrote version 1, so the engine tells them apart by the chunks the materials carry: density came with Clear
/// Sky and the multiplayer shoot factor with Call of Pripyat. Ordered as the engine compares them, and the game reads
/// ammunition, artefacts, bone protection and stalker fire differently by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameMtlVersion {
  /// `GAMEMTL_VERSION_SOC`.
  ShadowOfChernobyl,
  /// `GAMEMTL_VERSION_CS`.
  ClearSky,
  /// `GAMEMTL_VERSION_COP`.
  CallOfPripyat,
}
