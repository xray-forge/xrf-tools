use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_dds::{DdsEncoding, DdsMipChain, ImageFormat, Quality, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_level::{BuildCformFile, BuildPrjFile, LevelFile, LevelGeomFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::{StagedFile, format_path};

use crate::chart::lightmap_faces::LightmapFaces;
use crate::compiler_texture::CompilerTexture;
use crate::compiler_textures::CompilerTextures;
use crate::geometry::geometry_level::GeometryLevel;
use crate::geometry::geometry_level_request::GeometryLevelRequest;
use crate::level_compile_request::LevelCompileRequest;
use crate::level_compile_step_run::LevelCompileStepRun;
use crate::light::light_bake::LightBake;
use crate::light::light_bake_request::LightBakeRequest;

/// A compile's geometry step by our compiler, xrLC's part ([`GeometryLevel`]), with the light step where the compile
/// runs it: the level's `level`, `level.geom`, `level.geomx`, `level.cform`, `build.cform`, `build.lights`, each
/// lightmap page's pair and each implicitly lit texture's maps, from its `build.prj`, the game data's
/// `shaders_xrlc.xr` and the textures under the request's roots.
pub(crate) struct GeometryStep;

impl GeometryStep {
  /// `level.cform`.
  const COLLISION: &'static str = "level.cform";

  /// Compiles the request's level and writes what xrLC writes, its light baked where `is_lit` (the light step runs with
  /// it), else neutral; with the report, how the light was baked.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable, a phase refuses the project, the bake fails, the job is cancelled,
  /// or a file is not written.
  pub(crate) fn run(
    request: &LevelCompileRequest,
    job: &JobHandle,
    is_lit: bool,
  ) -> XrfResult<(LevelCompileStepRun, Option<LightBake>)> {
    let level: &Path = &request.level;
    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildPrjFile::FILE_NAME))?;
    let library: ShaderCompilerFile =
      ShaderCompilerFile::read_from_path::<XRayByteOrder, _>(&request.gamedata.join(ShaderCompilerFile::FILE_NAME))?;
    let textures: Vec<CompilerTexture> = CompilerTexture::resolve_all(&project, &request.textures)?;

    job.check_cancelled()?;

    let compiled: GeometryLevel = GeometryLevel::compile(&GeometryLevelRequest {
      project: &project,
      library: &library,
      textures: &textures,
      roots: &request.textures,
      quality: request.light_quality,
      light: is_lit.then_some(LightBakeRequest {
        backend: request.backend,
        dxc: request.dxc.as_deref(),
        gpu: request.gpu,
      }),
    })?;

    job.check_cancelled()?;

    let collision = compiled.get_collision_file()?;

    Self::write(&level.join(LevelFile::FILE_NAME), |writer| {
      compiled.bundle.write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(LevelGeomFile::FILE_NAME), |writer| {
      compiled.geometry.write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(LevelGeomFile::DETAIL_FILE_NAME), |writer| {
      compiled.fast.write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(Self::COLLISION), |writer| {
      collision.write_with_geometry::<XRayByteOrder>(&compiled.collision, writer)
    })?;
    Self::write(&level.join(BuildCformFile::FILE_NAME), |writer| {
      compiled.ray_model.write::<XRayByteOrder>(writer)
    })?;
    Self::write(&level.join(LevelLightsFile::FILE_NAME), |writer| {
      compiled.lights.write::<XRayByteOrder>(writer)
    })?;

    let side: u32 = compiled.lightmap_side;

    for (page, textures) in compiled.lightmaps.iter().enumerate() {
      for (name, texels) in LightmapFaces::get_textures(page as u32).iter().zip(textures) {
        Self::write_texture(&level.join(format!("{name}.dds")), texels, side, side)?;
      }
    }

    for map in &compiled.implicit_maps {
      let path: PathBuf = CompilerTextures::get_path(level, &map.name, "dds");
      let lightmap: PathBuf = CompilerTextures::get_path(level, &format!("{}_lm", map.name), "dds");

      if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|error| {
          XrfError::new_io_error(format!("{} was not made: {error}", format_path(folder)), error.kind())
        })?;
      }

      Self::write_texture(&path, &map.base, map.width, map.height)?;
      Self::write_texture(&lightmap, &map.lightmap, map.width, map.height)?;
    }

    Ok((
      LevelCompileStepRun::GeometryCompiler {
        census: compiled.census,
      },
      compiled.light,
    ))
  }

  /// Writes what `fill` puts in a writer to `path`, staged so a failed write leaves the file as it was.
  fn write(path: &Path, fill: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    fill(&mut writer)?;

    let bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    StagedFile::write(path, &bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(path)), error.kind()))
  }

  /// A lightmap texture of `texels`, `width` by `height`, as BC3 with no mips (`CLightmap::Save`'s and the implicit
  /// maps' DXT5).
  fn write_texture(path: &Path, texels: &[[u8; 4]], width: u32, height: u32) -> XrfResult {
    let image: RgbaImage = RgbaImage::from_raw(width, height, texels.as_flattened().to_vec())
      .ok_or_else(|| XrfError::new_unexpected_error("A lightmap texture holds as many texels as its size"))?;

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Normal)
      .encode(&DdsMipChain::from_levels(vec![image])?)?
      .write_to_path(path)
  }
}
