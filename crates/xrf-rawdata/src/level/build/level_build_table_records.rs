use xrf_level::{BuildPrjMaterial, BuildPrjShader, BuildPrjTexture};

/// The tables as `build.prj` stores them.
pub(crate) struct LevelBuildTableRecords {
  pub(crate) materials: Vec<BuildPrjMaterial>,
  pub(crate) render_shaders: Vec<BuildPrjShader>,
  pub(crate) compiler_shaders: Vec<BuildPrjShader>,
  pub(crate) textures: Vec<BuildPrjTexture>,
}
