use std::path::Path;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_dds::{DdsEncoding, DdsMipChain, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_level::{BuildCformFile, BuildPrjFile, LevelFile, LevelGeomFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::{StagedFile, format_path};

use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_draft_light::GeometryDraftLight;
use crate::geometry::geometry_level::GeometryLevel;
use crate::level_compile_request::LevelCompileRequest;
use crate::level_compile_step_run::LevelCompileStepRun;

/// A compile's geometry step by our compiler, xrLC's part with draft lightmaps ([`GeometryLevel`]): the level's `level`,
/// `level.geom`, `level.geomx`, `level.cform`, `build.cform`, `build.lights` and the draft lightmap pair, from its
/// `build.prj`, the game data's `shaders_xrlc.xr` and the textures under the request's roots.
pub(crate) struct GeometryStep;

impl GeometryStep {
  /// `level.cform`.
  const COLLISION: &'static str = "level.cform";

  /// Compiles the request's level and writes what xrLC writes.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable, a phase refuses the project, the job is cancelled, or a file is not
  /// written.
  pub(crate) fn run(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<LevelCompileStepRun> {
    let level: &Path = &request.level;
    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildPrjFile::FILE_NAME))?;
    let library: ShaderCompilerFile =
      ShaderCompilerFile::read_from_path::<XRayByteOrder, _>(&request.gamedata.join(ShaderCompilerFile::FILE_NAME))?;
    let textures: Vec<CompilerTexture> = CompilerTexture::resolve_all(&project, &request.textures)?;

    job.check_cancelled()?;

    let compiled: GeometryLevel = GeometryLevel::compile(&project, &library, &textures, &request.textures)?;

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

    if compiled.has_lightmaps {
      for (name, texel) in GeometryDraftLight::LIGHTMAP_TEXTURES
        .iter()
        .zip(GeometryLevel::get_lightmap_texels())
      {
        Self::write_lightmap(&level.join(format!("{name}.dds")), texel)?;
      }
    }

    Ok(LevelCompileStepRun::GeometryCompiler {
      census: compiled.census,
    })
  }

  /// Writes what `fill` puts in a writer to `path`, staged so a failed write leaves the file as it was.
  fn write(path: &Path, fill: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    fill(&mut writer)?;

    let bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    StagedFile::write(path, &bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(path)), error.kind()))
  }

  /// A draft lightmap of one texel colour, [`GeometryDraftLight::LIGHTMAP_SIDE`] a side, as BC3, which carries a
  /// uniform colour and alpha exactly.
  fn write_lightmap(path: &Path, texel: [u8; 4]) -> XrfResult {
    let side: u32 = GeometryDraftLight::LIGHTMAP_SIDE;
    let image: RgbaImage = RgbaImage::from_pixel(side, side, Rgba(texel));

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
      .encode(&DdsMipChain::from_levels(vec![image])?)?
      .write_to_path(path)
  }
}
