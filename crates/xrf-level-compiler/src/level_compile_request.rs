use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::level_compile_step::LevelCompileStep;

/// A level's compile, as a front-end asks for it: the level's directory the build wrote into, the steps to run, and
/// what the SDK's compilers need to run them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileRequest {
  /// The level's directory, `<levels>/<level>`, holding what `level build` wrote; the compile writes beside it.
  pub level: PathBuf,
  /// The steps to run, each once in [`LevelCompileStep::ALL`]'s order whatever the order given; every step when empty.
  pub steps: Vec<LevelCompileStep>,
  /// The SDK's `editors` directory, whose `bin` holds xrLC, xrDO_Light and xrAI.
  pub sdk: PathBuf,
  /// The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`.
  pub gamedata: PathBuf,
  /// The textures root (`$game_textures$`): a `.thm` for every texture `build.prj` names, and the image where the
  /// texture has alpha or is lit implicitly.
  pub textures: PathBuf,
  /// The directory the compilers run in, holding the `fsgame.ltx` written for them and their logs; a directory of the
  /// system's temporary one named after the level by default.
  pub work: Option<PathBuf>,
}
