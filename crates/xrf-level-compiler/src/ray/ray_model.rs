use std::path::Path;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_geometry::BvhTracer;
use xrf_level::{BuildCformFile, BuildPrjFile};
use xrf_shaders::ShaderCompilerFile;

use crate::level_compile_request::LevelCompileRequest;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;

/// What a compile's steps trace their rays through: xrLC's ray model, `build.cform`, in a tracer, and how each of its
/// faces meets a ray by the build's `build.prj`, the game data's `shaders_xrlc.xr` and the textures under the request's
/// roots.
pub(crate) struct RayModel {
  pub(crate) tracer: BvhTracer,
  pub(crate) shadows: ShadowMaterials,
}

impl RayModel {
  /// The request's level's ray model.
  ///
  /// # Errors
  ///
  /// Returns an error when an input is unreadable or the shadow materials cannot be loaded (`ShadowMaterials::load`).
  pub(crate) fn read(request: &LevelCompileRequest) -> XrfResult<Self> {
    let level: &Path = &request.level;
    let cform: BuildCformFile =
      BuildCformFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildCformFile::FILE_NAME))?;
    let project: BuildPrjFile = BuildPrjFile::read_from_path::<XRayByteOrder, _>(&level.join(BuildPrjFile::FILE_NAME))?;
    let library: ShaderCompilerFile =
      ShaderCompilerFile::read_from_path::<XRayByteOrder, _>(&request.gamedata.join(ShaderCompilerFile::FILE_NAME))?;
    let shadows: ShadowMaterials = ShadowMaterials::load(&ShadowMaterialsRequest {
      project: &project,
      faces: &cform.raycast_faces,
      library: &library,
      textures: &request.textures,
    })?;

    Ok(Self {
      tracer: cform.build_tracer(),
      shadows,
    })
  }
}
