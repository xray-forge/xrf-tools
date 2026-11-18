use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, ImageFormat, Quality, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_level::{BuildCformFile, BuildPrjFile, LevelCformFile, LevelFile, LevelGeomFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::OutputFiles;

use crate::compile::compiler_output::CompilerOutput;
use crate::compile::level_compile_request::LevelCompileRequest;
use crate::compile::level_compile_step_run::LevelCompileStepRun;
use crate::geometry::level::geometry_level::GeometryLevel;
use crate::geometry::level::geometry_level_request::GeometryLevelRequest;
use crate::geometry::level::geometry_step_run::GeometryStepRun;
use crate::input::compiler_texture::CompilerTexture;
use crate::input::compiler_textures::CompilerTextures;
use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::ray::ray_tracing::RayTracing;

/// A compile's geometry step by our compiler, xrLC's part ([`GeometryLevel`]), with the light step where the compile
/// runs it: the level's `level`, `level.geom`, `level.geomx`, `level.cform`, `build.cform`, `build.lights`, each
/// lightmap page's pair and each implicitly lit texture's maps, from its `build.prj`, the game data's
/// `shaders_xrlc.xr` and the textures under the request's roots.
pub(crate) struct GeometryStep;

impl GeometryStep {
  /// The files the step always writes in the level's directory.
  pub(crate) const OUTPUTS: [&'static str; 6] = [
    LevelFile::FILE_NAME,
    LevelGeomFile::FILE_NAME,
    LevelGeomFile::DETAIL_FILE_NAME,
    LevelCformFile::FILE_NAME,
    BuildCformFile::FILE_NAME,
    LevelLightsFile::FILE_NAME,
  ];

  /// The prefix of every lightmap page's file, as many as the compile bakes.
  pub(crate) const LIGHTMAP_FAMILY: &'static str = "lmap#";

  /// The textures an implicitly lit texture's maps are written as below the level's directory, each a `.dds`: its
  /// base with the hemisphere in alpha, then its lightmap.
  pub(crate) fn get_implicit_textures(name: &str) -> [String; 2] {
    [String::from(name), format!("{name}_lm")]
  }

  /// Compiles the request's level and writes what xrLC writes, its light baked with its rays traced by `light` where
  /// the light step runs with it, else neutral; with the report, how the light was baked.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable, a phase refuses the project, the bake fails, the job is cancelled,
  /// or a file is not written.
  pub(crate) fn run(
    request: &LevelCompileRequest,
    job: &JobHandle,
    light: Option<RayTracing<'_>>,
  ) -> XrfResult<GeometryStepRun> {
    let level: &Path = &request.level;
    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildPrjFile::FILE_NAME))?;
    let library: ShaderCompilerFile =
      ShaderCompilerFile::read_from_path::<XRayByteOrder, _>(&request.gamedata.join(ShaderCompilerFile::FILE_NAME))?;
    let roots: Vec<PathBuf> = request.get_textures_roots();
    let textures: Vec<CompilerTexture> = CompilerTexture::resolve_all(&project, &roots)?;

    job.check_cancelled()?;

    let compiled: GeometryLevel = GeometryLevel::compile(&GeometryLevelRequest {
      project: &project,
      library: &library,
      textures: &textures,
      roots: &roots,
      quality: request.light_quality,
      light,
    })?;

    job.check_cancelled()?;

    let collision = compiled.get_collision_file()?;

    CompilerOutput::write_chunks(&level.join(LevelFile::FILE_NAME), |writer| {
      compiled.bundle.write::<XRayByteOrder>(writer)
    })?;
    CompilerOutput::write_chunks(&level.join(LevelGeomFile::FILE_NAME), |writer| {
      compiled.geometry.write::<XRayByteOrder>(writer)
    })?;
    CompilerOutput::write_chunks(&level.join(LevelGeomFile::DETAIL_FILE_NAME), |writer| {
      compiled.fast.write::<XRayByteOrder>(writer)
    })?;
    CompilerOutput::write_chunks(&level.join(LevelCformFile::FILE_NAME), |writer| {
      collision.write_with_geometry::<XRayByteOrder>(&compiled.collision, writer)
    })?;
    CompilerOutput::write_chunks(&level.join(BuildCformFile::FILE_NAME), |writer| {
      compiled.ray_model.write::<XRayByteOrder>(writer)
    })?;
    CompilerOutput::write_chunks(&level.join(LevelLightsFile::FILE_NAME), |writer| {
      compiled.lights.write::<XRayByteOrder>(writer)
    })?;

    let side: u32 = compiled.lightmaps.side;
    let mut pages: Vec<PathBuf> = Vec::with_capacity(compiled.lightmaps.pages.len() * 2);

    for (page, pair) in (0..).zip(&compiled.lightmaps.pages) {
      for (name, texels) in LightmapFaces::get_textures(page)
        .iter()
        .zip([&pair.first, &pair.second])
      {
        let path: PathBuf = level.join(format!("{name}.dds"));

        Self::write_texture(&path, texels, side, side)?;
        pages.push(path);
      }
    }

    // A compile baking fewer pages than an earlier one leaves that one's last pages behind, which the game would load.
    OutputFiles::new(level)
      .with_family(Self::LIGHTMAP_FAMILY)
      .remove_stale(&pages)?;

    for map in &compiled.implicit_maps {
      for (name, texels) in Self::get_implicit_textures(&map.name)
        .iter()
        .zip([&map.base, &map.lightmap])
      {
        Self::write_texture(
          &CompilerTextures::get_path(level, name, "dds"),
          texels,
          map.width,
          map.height,
        )?;
      }
    }

    Ok(GeometryStepRun {
      run: LevelCompileStepRun::GeometryCompiler {
        census: compiled.census,
      },
      bake: compiled.light,
    })
  }

  /// A lightmap texture of `texels`, `width` by `height`, as BC3 with no mips (`CLightmap::Save`'s and the implicit
  /// maps' DXT5), its folder made where it is missing.
  fn write_texture(path: &Path, texels: &[[u8; 4]], width: u32, height: u32) -> XrfResult {
    let image: RgbaImage = RgbaImage::from_raw(width, height, texels.as_flattened().to_vec())
      .ok_or_else(|| XrfError::new_unexpected_error("A lightmap texture holds as many texels as its size"))?;

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Normal)
      .encode(&DdsMipChain::from_levels(vec![image])?)?
      .write_to_path(path)
  }
}
