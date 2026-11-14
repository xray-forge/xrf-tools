use std::path::PathBuf;

use xrf_level::BuildPrjFile;
use xrf_shaders::ShaderCompilerFile;

use crate::compiler_texture::CompilerTexture;
use crate::light::light_bake_request::LightBakeRequest;
use crate::light::light_quality::LightQuality;

/// What [`crate::GeometryLevel::compile`] compiles a level from, each already read, and whether and how it lights it.
#[derive(Clone, Copy)]
pub struct GeometryLevelRequest<'a> {
  /// The level's `build.prj`.
  pub project: &'a BuildPrjFile,
  /// The game data's compiler shaders, `shaders_xrlc.xr`.
  pub library: &'a ShaderCompilerFile,
  /// Each `build.prj` texture as xrLC loads it.
  pub textures: &'a [CompilerTexture],
  /// The textures roots the alpha images and the implicitly lit textures are read from.
  pub roots: &'a [PathBuf],
  /// How finely the level is lit, which also decides the sky and sun `build.lights` holds.
  pub quality: LightQuality,
  /// How the light is baked; none for the neutral light the renderers light over in real time
  /// ([`crate::GeometryDraftLight`]).
  pub light: Option<LightBakeRequest<'a>>,
}
