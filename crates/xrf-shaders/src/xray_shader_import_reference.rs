use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::ShaderRenderer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayShaderImportReference {
  line_number: usize,
  path: String,
}

impl XRayShaderImportReference {
  pub fn parse_all(path: &Path, renderer: ShaderRenderer, source: &[u8]) -> XrfResult<Vec<Self>> {
    match renderer {
      ShaderRenderer::DirectX11 => Self::parse_directx11(path, source),
      ShaderRenderer::OpenGl => Self::parse_open_gl(path, source),
    }
  }

  fn parse_directx11(path: &Path, source: &[u8]) -> XrfResult<Vec<Self>> {
    let mut imports: Vec<Self> = Vec::new();

    for (index, line) in String::from_utf8_lossy(source).lines().enumerate() {
      let line_number: usize = index + 1;
      let line: &str = line.trim_start();

      let Some(include) = line.strip_prefix('#') else {
        continue;
      };

      let include: &str = include.trim_start();

      let Some(include) = include.strip_prefix("include") else {
        continue;
      };
      let include: &str = include.trim_start();

      let Some((include, closing_delimiter)) = (match include.chars().next() {
        Some('"') => Some((&include[1..], '"')),
        Some('<') => Some((&include[1..], '>')),
        _ => None,
      }) else {
        return Err(XrfError::new_invalid_error(format!(
          "Shader {} has malformed #include on line {line_number}: expected a quoted or angle-bracket import path",
          format_path(path)
        )));
      };

      imports.push(Self::from_include_path(path, line_number, include, closing_delimiter)?);
    }

    Ok(imports)
  }

  fn parse_open_gl(path: &Path, source: &[u8]) -> XrfResult<Vec<Self>> {
    let mut imports: Vec<Self> = Vec::new();

    for (index, line) in String::from_utf8_lossy(source).lines().enumerate() {
      let line_number: usize = index + 1;
      let mut remainder: &str = line;

      while let Some(include_offset) = remainder.find("#include") {
        let include: &str = &remainder[include_offset + "#include".len()..];
        let Some(include) = include.trim_start().strip_prefix('"') else {
          return Err(XrfError::new_invalid_error(format!(
            "OpenGL shader {} has malformed #include on line {line_number}: expected a quoted import path",
            format_path(path)
          )));
        };

        let import: Self = Self::from_include_path(path, line_number, include, '"')?;
        let import_path_length: usize = import.path.len();
        remainder = &include[import_path_length + 1..];
        imports.push(import);
      }
    }

    Ok(imports)
  }

  fn from_include_path(
    shader_path: &Path,
    line_number: usize,
    include: &str,
    closing_delimiter: char,
  ) -> XrfResult<Self> {
    let Some(end) = include.find(closing_delimiter) else {
      return Err(XrfError::new_invalid_error(format!(
        "Shader {} has malformed #include on line {line_number}: expected a closing {closing_delimiter}",
        format_path(shader_path)
      )));
    };

    let import_path: &str = &include[..end];

    if import_path.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Shader {} has malformed #include on line {line_number}: import path is empty",
        format_path(shader_path)
      )));
    }

    Ok(Self {
      line_number,
      path: import_path.to_string(),
    })
  }

  pub fn line_number(&self) -> usize {
    self.line_number
  }

  pub fn path(&self) -> &str {
    &self.path
  }

  pub fn into_path(self) -> String {
    self.path
  }
}
