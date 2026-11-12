use std::path::PathBuf;

use xrf_level::{BuildCformRaycastFace, BuildPrjFile};
use xrf_shaders::ShaderCompilerFile;

/// What [`crate::ShadowMaterials::load`] reads a level's shadow materials from: files already read, and the texture
/// roots it reads each texture's `.thm` and alpha image under.
#[derive(Clone, Copy, Debug)]
pub struct ShadowMaterialsRequest<'a> {
  /// The scene's project, `build.prj`: its materials, compiler shaders and textures.
  pub project: &'a BuildPrjFile,
  /// The ray model's faces, `build.cform`'s, in the order its tracer is built over them.
  pub faces: &'a [BuildCformRaycastFace],
  /// The compiler shader library, `shaders_xrlc.xr`.
  pub library: &'a ShaderCompilerFile,
  /// The texture roots (`$game_textures$`), searched in order for each file, the first holding it winning.
  pub textures: &'a [PathBuf],
}
