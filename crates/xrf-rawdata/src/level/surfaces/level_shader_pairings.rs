use std::collections::HashMap;

use xrf_editable_object::EditableObjectFile;

/// How often an object library's surfaces pair each engine shader with each compiler shader, which breaks ties between
/// compiler shaders that fit a compiled surface.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelShaderPairings {
  /// Surfaces by engine shader, then by compiler shader, both lowercased.
  counts: HashMap<String, HashMap<String, usize>>,
}

impl LevelShaderPairings {
  /// Counts every surface of `object`.
  pub fn add_object(&mut self, object: &EditableObjectFile) {
    for surface in object.surfaces.as_deref().unwrap_or_default() {
      self.add(&surface.shader, &surface.compiler_shader);
    }
  }

  /// Counts one surface of `engine` shader paired with `compiler` shader.
  pub fn add(&mut self, engine: &str, compiler: &str) {
    *self
      .counts
      .entry(engine.to_lowercase())
      .or_default()
      .entry(compiler.to_lowercase())
      .or_default() += 1;
  }

  /// How many surfaces paired `engine` with `compiler`.
  pub fn get(&self, engine: &str, compiler: &str) -> usize {
    self
      .counts
      .get(&engine.to_lowercase())
      .and_then(|compilers| compilers.get(&compiler.to_lowercase()))
      .copied()
      .unwrap_or(0)
  }
}
