use serde::Serialize;
use xrf_db::ShaderCompilerShader;

/// One compiler shader, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShaderCompilerShader {
  /// The name a surface declares, which the renderer's own blender library answers under too.
  pub name: String,
  /// What the compiler is told to do with the surface, named.
  pub flags: Vec<String>,
  pub vertex_translucency: f32,
  pub vertex_ambient: f32,
  /// Lightmap texels per unit, which is what a surface costs to bake.
  pub lightmap_density: f32,
}

impl ArchiveShaderCompilerShader {
  /// Every shader of a library, in the order it numbers them.
  pub fn of_all(shaders: &[ShaderCompilerShader]) -> Vec<Self> {
    shaders.iter().map(Self::of).collect()
  }

  /// One shader, taken over what it tells the compiler.
  fn of(shader: &ShaderCompilerShader) -> Self {
    Self {
      name: shader.name.clone(),
      flags: shader.get_named_flags().into_iter().map(ToOwned::to_owned).collect(),
      vertex_translucency: shader.vertex_translucency,
      vertex_ambient: shader.vertex_ambient,
      lightmap_density: shader.lightmap_density,
    }
  }
}
