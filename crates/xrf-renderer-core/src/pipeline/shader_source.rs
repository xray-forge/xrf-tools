use xrf_error::XrfResult;

/// Where a pipeline cache reads its WGSL from: a module by name, its imports resolved.
pub trait ShaderSource {
  /// The module's whole source.
  ///
  /// # Errors
  ///
  /// Returns an error for a module, or an import of it, the source does not have.
  fn compose(&self, module: &str) -> XrfResult<String>;
}
