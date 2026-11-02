use serde::Serialize;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_shaders::ShaderCompilerFile;

use crate::describe::shader_compiler::archive_shader_compiler_shader::ArchiveShaderCompilerShader;
use crate::describe::source::ArchiveDescribeSource;

/// Everything the viewer says about the compiler shader library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShaderCompilerDescription {
  pub shaders: Vec<ArchiveShaderCompilerShader>,
}

impl ArchiveShaderCompilerDescription {
  /// Reads the compiler shader library an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a compiler library this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: ShaderCompilerFile = ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      shaders: ArchiveShaderCompilerShader::of_all(&file.shaders),
    })
  }
}
