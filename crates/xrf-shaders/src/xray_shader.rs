use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::xray_shader_import_reference::XRayShaderImportReference;
use crate::{ShaderRenderer, XRayShaderCompiler, XRayShaderImport, XRayShaderSourceLoader};

/// A fully resolved X-Ray shader source file and its nested imports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayShader {
  imports: Vec<XRayShaderImport>,
  path: PathBuf,
  source: Vec<u8>,
}

impl XRayShader {
  /// Load a shader and recursively resolve all of its renderer-specific
  /// imports through the provided source loader.
  pub fn load<P, L>(path: P, renderer: ShaderRenderer, shaders_root: &Path, loader: &L) -> XrfResult<Self>
  where
    P: AsRef<Path>,
    L: XRayShaderSourceLoader,
  {
    let mut active_paths: Vec<PathBuf> = Vec::new();

    Self::load_from_path(path.as_ref(), renderer, shaders_root, loader, &mut active_paths)
  }

  /// Compile this resolved shader through a renderer-specific compiler.
  pub fn compile<C>(&self, renderer: ShaderRenderer, compiler: &C) -> XrfResult
  where
    C: XRayShaderCompiler,
  {
    compiler.compile(self, renderer)
  }

  pub fn imports(&self) -> &[XRayShaderImport] {
    &self.imports
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn source(&self) -> &[u8] {
    &self.source
  }

  fn load_from_path<L>(
    path: &Path,
    renderer: ShaderRenderer,
    shaders_root: &Path,
    loader: &L,
    active_paths: &mut Vec<PathBuf>,
  ) -> XrfResult<Self>
  where
    L: XRayShaderSourceLoader,
  {
    let Some(source) = loader.load_source(path)? else {
      return Err(XrfError::new_not_found_error(format!(
        "Shader source was not found: {}",
        format_path(path)
      )));
    };

    Self::load_from_source(path, source, renderer, shaders_root, loader, active_paths)
  }

  fn load_from_source<L>(
    path: &Path,
    source: Vec<u8>,
    renderer: ShaderRenderer,
    shaders_root: &Path,
    loader: &L,
    active_paths: &mut Vec<PathBuf>,
  ) -> XrfResult<Self>
  where
    L: XRayShaderSourceLoader,
  {
    if active_paths.iter().any(|active_path| active_path == path) {
      return Err(XrfError::new_verify_error(format!(
        "Shader include cycle reaches {}",
        format_path(path)
      )));
    }

    active_paths.push(path.to_path_buf());

    let imports: Vec<XRayShaderImportReference> = XRayShaderImportReference::parse_all(path, renderer, &source)?;
    let mut resolved_imports: Vec<XRayShaderImport> = Vec::with_capacity(imports.len());

    for import in imports {
      let shader: XRayShader = Self::load_import(path, &import, renderer, shaders_root, loader, active_paths)?;

      resolved_imports.push(XRayShaderImport::from_reference(import, shader));
    }

    active_paths.pop();

    Ok(Self {
      imports: resolved_imports,
      path: path.to_path_buf(),
      source,
    })
  }

  fn load_import<L>(
    source_path: &Path,
    import: &XRayShaderImportReference,
    renderer: ShaderRenderer,
    shaders_root: &Path,
    loader: &L,
    active_paths: &mut Vec<PathBuf>,
  ) -> XrfResult<Self>
  where
    L: XRayShaderSourceLoader,
  {
    for candidate in renderer.include_candidate_paths(shaders_root, import.path()) {
      if let Some(source) = loader.load_source(&candidate)? {
        return Self::load_from_source(&candidate, source, renderer, shaders_root, loader, active_paths);
      }
    }

    Err(XrfError::new_not_found_error(format!(
      "Shader {} includes missing file '{}' on line {}",
      format_path(source_path),
      import.path(),
      import.line_number()
    )))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};

  use xrf_error::{XrfError, XrfResult};

  use super::XRayShader;
  use crate::{ShaderRenderer, XRayShaderPlaceholderCompiler, XRayShaderSourceLoader};

  struct TestSourceLoader {
    sources: HashMap<PathBuf, Vec<u8>>,
  }

  impl TestSourceLoader {
    fn with_sources(sources: impl IntoIterator<Item = (PathBuf, Vec<u8>)>) -> Self {
      Self {
        sources: sources.into_iter().collect(),
      }
    }
  }

  impl XRayShaderSourceLoader for TestSourceLoader {
    fn load_source(&self, path: &Path) -> XrfResult<Option<Vec<u8>>> {
      Ok(self.sources.get(path).cloned())
    }
  }

  #[test]
  fn loads_nested_renderer_imports() -> XrfResult {
    let root: &Path = Path::new("shaders");
    let main_path: PathBuf = root.join("r3/main.ps");
    let common_path: PathBuf = root.join("r3/common.h");
    let loader: TestSourceLoader = TestSourceLoader::with_sources([
      (main_path.clone(), b"#include \"common.h\"\n".to_vec()),
      (common_path.clone(), b"float value;\n".to_vec()),
    ]);

    let shader: XRayShader = XRayShader::load(&main_path, ShaderRenderer::DirectX11, root, &loader)?;

    assert_eq!(shader.path(), main_path);
    assert_eq!(shader.imports().len(), 1);
    assert_eq!(shader.imports()[0].path(), "common.h");
    assert_eq!(shader.imports()[0].shader().path(), common_path);

    Ok(())
  }

  #[test]
  fn supports_directx_root_include_fallback() -> XrfResult {
    let root: &Path = Path::new("shaders");
    let main_path: PathBuf = root.join("r3/main.ps");
    let common_path: PathBuf = root.join("shared/common.h");
    let loader: TestSourceLoader = TestSourceLoader::with_sources([
      (main_path.clone(), b"#include \"shared/common.h\"\n".to_vec()),
      (common_path.clone(), b"float value;\n".to_vec()),
    ]);

    let shader: XRayShader = XRayShader::load(&main_path, ShaderRenderer::DirectX11, root, &loader)?;

    assert_eq!(shader.imports()[0].shader().path(), common_path);

    Ok(())
  }

  #[test]
  fn follows_renderer_specific_include_rules() -> XrfResult {
    let root: &Path = Path::new("shaders");
    let directx_path: PathBuf = root.join("r3/directx.ps");
    let directx_common_path: PathBuf = root.join("r3/common.h");
    let opengl_path: PathBuf = root.join("gl/opengl.ps");
    let loader: TestSourceLoader = TestSourceLoader::with_sources([
      (
        directx_path.clone(),
        b"//#include \"commented.h\"\n# include <common.h>\n".to_vec(),
      ),
      (directx_common_path.clone(), b"float value;\n".to_vec()),
      (opengl_path.clone(), b"//#include \"commented.h\"\n".to_vec()),
    ]);

    let directx_shader: XRayShader = XRayShader::load(&directx_path, ShaderRenderer::DirectX11, root, &loader)?;
    let opengl_result = XRayShader::load(&opengl_path, ShaderRenderer::OpenGl, root, &loader);

    assert_eq!(directx_shader.imports().len(), 1);
    assert_eq!(directx_shader.imports()[0].shader().path(), directx_common_path);
    assert!(matches!(opengl_result, Err(XrfError::NotFound { .. })));

    Ok(())
  }

  #[test]
  fn reports_invalid_imports_and_cycles() {
    let root: &Path = Path::new("shaders");
    let invalid_path: PathBuf = root.join("gl/invalid.ps");
    let first_path: PathBuf = root.join("gl/first.h");
    let second_path: PathBuf = root.join("gl/second.h");
    let loader: TestSourceLoader = TestSourceLoader::with_sources([
      (invalid_path.clone(), b"#include invalid.h\n".to_vec()),
      (first_path.clone(), b"#include \"second.h\"\n".to_vec()),
      (second_path, b"#include \"first.h\"\n".to_vec()),
    ]);

    let invalid_result = XRayShader::load(&invalid_path, ShaderRenderer::OpenGl, root, &loader);
    let cycle_result = XRayShader::load(&first_path, ShaderRenderer::OpenGl, root, &loader);

    assert!(matches!(invalid_result, Err(XrfError::Invalid { .. })));
    assert!(matches!(cycle_result, Err(XrfError::Verify { .. })));
  }

  #[test]
  fn compiler_placeholder_reports_that_compilation_is_not_implemented() -> XrfResult {
    let root: &Path = Path::new("shaders");
    let path: PathBuf = root.join("r3/main.ps");
    let loader: TestSourceLoader = TestSourceLoader::with_sources([(path.clone(), b"float value;\n".to_vec())]);
    let shader: XRayShader = XRayShader::load(&path, ShaderRenderer::DirectX11, root, &loader)?;
    let result = shader.compile(ShaderRenderer::DirectX11, &XRayShaderPlaceholderCompiler);

    assert!(matches!(result, Err(XrfError::NotImplemented { .. })));

    Ok(())
  }
}
