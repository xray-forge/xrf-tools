use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ai::ai_map_covers::AiMapCovers;
use crate::level_compile_step::LevelCompileStep;
use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;

/// A level's compile, as a front-end asks for it: the level's directory the build wrote into, the steps to run, and
/// what they read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileRequest {
  /// The level's directory, `<levels>/<level>`, holding what `level build` wrote; the compile writes beside it.
  pub level: PathBuf,
  /// The steps to run, each once in [`LevelCompileStep::ALL`]'s order whatever the order given; every step when empty.
  pub steps: Vec<LevelCompileStep>,
  /// The SDK's `editors` directory, whose `bin` holds xrLC and xrAI; needed only by the steps the SDK still runs
  /// ([`LevelCompileStep::is_sdk`]).
  pub sdk: Option<PathBuf>,
  /// The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`.
  pub gamedata: PathBuf,
  /// The textures roots (`$game_textures$`), searched in order for each file: a `.thm` for every texture `build.prj`
  /// names, and the image where the texture has alpha or is lit implicitly. The SDK's compilers take exactly one.
  pub textures: Vec<PathBuf>,
  /// The directory the SDK's compilers run in, holding the `fsgame.ltx` written for them and their logs; a directory of
  /// the system's temporary one named after the level by default.
  pub work: Option<PathBuf>,
  /// Whether a step of ours overwrites a file it writes where the level's directory holds it, rather than refusing.
  pub is_forced: bool,
  /// What traces the rays: the details step's shadow rays and the AI step's corner rays.
  pub backend: RayBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<PathBuf>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<RayGpu>,
  /// The covers the AI step computes: full ones traced against `build.cform`, or none, needing no `build.cform`.
  pub ai_covers: AiMapCovers,
}
