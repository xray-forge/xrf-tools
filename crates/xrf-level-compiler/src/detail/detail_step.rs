use std::path::Path;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::BvhTracer;
use xrf_job::JobHandle;
use xrf_level::{BuildCformFile, BuildPrjFile, LevelDetailsFile, LevelLightsFile};
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::{StagedFile, format_path};

use crate::detail::detail_light_request::DetailLightRequest;
use crate::detail::detail_lighter::DetailLighter;
use crate::detail::detail_lighting::DetailLighting;
use crate::detail::detail_slot_counts::DetailSlotCounts;
use crate::level_compile_request::LevelCompileRequest;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;

/// A compile's details step, xrDO_Light's part: the level's `level.details` written as `build.details` lit by
/// [`DetailLighter`] from xrLC's `build.cform` and `build.lights`, the build's `build.prj` and the game data's
/// `shaders_xrlc.xr`, the textures read under the request's roots.
pub(crate) struct DetailStep;

impl DetailStep {
  /// The details as the Build exports them, unlit, which the step reads.
  pub(crate) const SOURCE: &'static str = "build.details";

  /// The step's one output, which it refuses to overwrite unless the request is forced.
  pub(crate) const OUTPUT: &'static str = LevelDetailsFile::FILE_NAME;

  /// Lights the request's level's details and writes them, answering how the slots were met.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable, the shadow materials cannot be loaded (`ShadowMaterials::load`),
  /// the job is cancelled, or `level.details` is not written; a file left as it was in each case.
  pub(crate) fn run(request: &LevelCompileRequest, job: &JobHandle) -> XrfResult<DetailSlotCounts> {
    let level: &Path = &request.level;
    let details: LevelDetailsFile = LevelDetailsFile::read_from_path::<XRayByteOrder, _>(&level.join(Self::SOURCE))?;
    let cform: BuildCformFile =
      BuildCformFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildCformFile::FILE_NAME))?;
    let lights: LevelLightsFile =
      LevelLightsFile::read_from_path::<XRayByteOrder, _>(&level.join(LevelLightsFile::FILE_NAME))?;
    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildPrjFile::FILE_NAME))?;
    let library: ShaderCompilerFile =
      ShaderCompilerFile::read_from_path::<XRayByteOrder, _>(&request.gamedata.join(ShaderCompilerFile::FILE_NAME))?;
    let shadows: ShadowMaterials = ShadowMaterials::load(&ShadowMaterialsRequest {
      project: &project,
      faces: &cform.raycast_faces,
      library: &library,
      textures: &request.textures,
    })?;
    let tracer: BvhTracer = cform.build_tracer();
    let lighting: DetailLighting = DetailLighter::light(
      &DetailLightRequest {
        details: &details,
        tracer: &tracer,
        shadows: &shadows,
        lights: &lights,
      },
      job,
    )?;
    let mut writer: ChunkWriter = ChunkWriter::new();
    let output = level.join(Self::OUTPUT);

    lighting.details.write::<XRayByteOrder>(&mut writer)?;
    StagedFile::write(&output, &writer.buffer).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(&output)),
        error.kind(),
      )
    })?;

    Ok(lighting.counts)
  }
}
