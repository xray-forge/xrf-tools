#![doc = include_str!("../README.md")]

mod compiler_table;
mod library;
mod script;
mod source;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::compiler_table::{ShaderCompilerFile, ShaderCompilerShader};
pub use crate::library::{
  ShaderBlender, ShaderBlenderClass, ShaderBlenderProperty, ShaderBlenderPropertyKind, ShaderBlenderPropertyValue,
  ShaderBlenderToken, ShaderLibraryFile,
};
pub use crate::script::{
  XRayShaderBlendFactor, XRayShaderPass, XRayShaderPassState, XRayShaderSampler, XRayShaderSamplerTexture,
  XRayShaderScript,
};
pub use crate::source::{
  ShaderRenderer, XRayShader, XRayShaderCompiler, XRayShaderImport, XRayShaderPlaceholderCompiler,
  XRayShaderSourceLoader,
};
