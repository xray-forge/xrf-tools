use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ai::ai_map_covers::AiMapCovers;
use crate::compile::level_compile_step::LevelCompileStep;
use crate::light::bake::light_quality::LightQuality;
use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;
use crate::ray::ray_tracing::RayTracing;

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
  /// The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`.
  pub gamedata: PathBuf,
  /// The textures roots (`$game_textures$`) searched before the game data's own `textures`, such as one a decompile
  /// wrote impostor strips into ([`Self::get_textures_roots`]).
  pub textures: Vec<PathBuf>,
  /// Whether a step overwrites a file it writes where the level's directory holds it, rather than refusing.
  pub is_forced: bool,
  /// What traces the rays: the light step's lightmap and implicit map shadow rays, the details step's shadow rays and
  /// the AI step's corner rays.
  pub backend: RayBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<PathBuf>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<RayGpu>,
  /// The covers the AI step computes: full ones traced against `build.cform`, or none, needing no `build.cform`.
  pub ai_covers: AiMapCovers,
  /// How finely the light step lights the level: by `build.prj`'s own settings, or a draft that regenerates the sky and
  /// sun and takes fewer samples. It also decides the sky and sun the geometry step writes to `build.lights`.
  pub light_quality: LightQuality,
}

impl LevelCompileRequest {
  /// The directory below the game data holding the game's own textures.
  pub const TEXTURES_DIRECTORY: &'static str = "textures";

  /// The textures roots the steps search in order for each file, a `.thm` for every texture `build.prj` names and the
  /// image where the texture has alpha or is lit implicitly: the roots named, then the game data's `textures` unless it
  /// is among them, so a root holding only what a decompile wrote finds every other texture in the game's.
  pub fn get_textures_roots(&self) -> Vec<PathBuf> {
    let game: PathBuf = self.gamedata.join(Self::TEXTURES_DIRECTORY);
    let mut roots: Vec<PathBuf> = self.textures.clone();

    if !roots.contains(&game) {
      roots.push(game);
    }

    roots
  }

  /// What traces the steps' rays, as the request names it.
  pub fn get_tracing(&self) -> RayTracing<'_> {
    RayTracing {
      backend: self.backend,
      dxc: self.dxc.as_deref(),
      gpu: self.gpu,
    }
  }
}
