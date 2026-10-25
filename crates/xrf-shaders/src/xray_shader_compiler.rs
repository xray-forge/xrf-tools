use xrf_error::XrfResult;

use crate::{ShaderRenderer, XRayShader};

/// Compiles a fully resolved X-Ray shader source tree for one renderer.
pub trait XRayShaderCompiler {
  fn compile(&self, shader: &XRayShader, renderer: ShaderRenderer) -> XrfResult;
}
