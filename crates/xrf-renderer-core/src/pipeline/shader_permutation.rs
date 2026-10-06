/// A shader's permutation domain, declared as a struct (`#[derive(ShaderPermutation)]`): each field a WGSL `override`
/// constant of its name. A permutation switches code, never bindings; a variant binding otherwise is another pass
/// parameter struct.
pub trait ShaderPermutation {
  /// The constants a pipeline of this permutation is made with, by name.
  fn list_constants(&self) -> Vec<(&'static str, f64)>;

  /// `override name: type;` for each field, for the shader to declare.
  fn get_wgsl_overrides() -> String;
}

/// The one permutation of a shader with none.
impl ShaderPermutation for () {
  fn list_constants(&self) -> Vec<(&'static str, f64)> {
    Vec::new()
  }

  fn get_wgsl_overrides() -> String {
    String::new()
  }
}
