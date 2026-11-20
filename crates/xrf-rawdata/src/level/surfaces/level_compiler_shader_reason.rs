use serde::{Deserialize, Serialize};

/// Why a compiler shader was chosen among those fitting a surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelCompilerShaderReason {
  /// The library's objects pair the surface's engine shader with it most often.
  Paired,
  /// No library object pairs the engine shader with any that fits, and it is the one GSC's objects use most for what
  /// the level shows.
  CommonestFit,
  /// The first in `shaders_xrlc.xr` that fits.
  FirstFit,
  /// None fits, so the editor's `default`.
  NoFit,
}
