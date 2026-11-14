use std::path::{Path, PathBuf};

use xrf_level::BuildPrjFile;
use xrf_shaders::ShaderCompilerFile;

use crate::compiler_texture::CompilerTexture;
use crate::light::light_quality::LightQuality;
use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;

/// What [`crate::GeometryLevel::compile`] compiles a level from, each already read, and how it lights it.
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
  /// How finely the level is lit.
  pub quality: LightQuality,
  /// What traces the lightmaps' shadow rays.
  pub backend: RayBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<&'a Path>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<RayGpu>,
}
