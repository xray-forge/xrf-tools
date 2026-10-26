use crate::source::{XRayShader, XRayShaderImportReference};

/// A resolved shader source imported by another shader source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayShaderImport {
  line_number: usize,
  path: String,
  shader: Box<XRayShader>,
}

impl XRayShaderImport {
  pub(crate) fn from_reference(reference: XRayShaderImportReference, shader: XRayShader) -> Self {
    Self {
      line_number: reference.line_number(),
      path: reference.into_path(),
      shader: Box::new(shader),
    }
  }

  pub fn line_number(&self) -> usize {
    self.line_number
  }

  pub fn path(&self) -> &str {
    &self.path
  }

  pub fn shader(&self) -> &XRayShader {
    &self.shader
  }
}
