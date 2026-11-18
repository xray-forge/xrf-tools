use std::path::PathBuf;

use xrf_job::JobHandle;
use xrf_level::BuildPrjFile;
use xrf_shaders::ShaderCompilerFile;

use crate::input::compiler_texture::CompilerTexture;
use crate::light::bake::light_quality::LightQuality;
use crate::ray::ray_tracing::RayTracing;

/// What [`GeometryLevel::compile`](crate::geometry::level::geometry_level::GeometryLevel::compile) compiles a level
/// from, each already read, and whether and how it lights it.
#[derive(Clone, Copy)]
pub(crate) struct GeometryLevelRequest<'a> {
  /// The level's `build.prj`.
  pub(crate) project: &'a BuildPrjFile,
  /// The game data's compiler shaders, `shaders_xrlc.xr`.
  pub(crate) library: &'a ShaderCompilerFile,
  /// Each `build.prj` texture as xrLC loads it.
  pub(crate) textures: &'a [CompilerTexture],
  /// The textures roots the alpha images and the implicitly lit textures are read from.
  pub(crate) roots: &'a [PathBuf],
  /// How finely the level is lit, which also decides the sky and sun `build.lights` holds.
  pub(crate) quality: LightQuality,
  /// What traces the baked light's lightmap and implicit map shadow rays; none for the neutral light the renderers
  /// light over in real time ([`LightDraft`](crate::light::light_draft::LightDraft)).
  pub(crate) light: Option<RayTracing<'a>>,
  /// The job the compile reports its phases, charts and implicit map rows to, and stops on when it is cancelled.
  pub(crate) job: &'a JobHandle,
}
