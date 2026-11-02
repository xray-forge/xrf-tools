use std::path::{Path, PathBuf};

/// A renderer backend supported by the XRF engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShaderRenderer {
  /// The DirectX 11 renderer, which loads sources from `shaders/r3`.
  DirectX11,
  /// The OpenGL renderer, which loads sources from `shaders/gl`.
  OpenGl,
}

impl ShaderRenderer {
  /// Directory selected by the engine under the gamedata `shaders` root.
  pub const fn directory_name(self) -> &'static str {
    match self {
      Self::DirectX11 => "r3",
      Self::OpenGl => "gl",
    }
  }

  /// Stable name used in diagnostics and user-facing summaries.
  pub const fn display_name(self) -> &'static str {
    match self {
      Self::DirectX11 => "DirectX 11",
      Self::OpenGl => "OpenGL",
    }
  }

  /// Ordered paths the engine tries for an import.
  ///
  /// This returns candidates only. The caller owns filesystem access and
  /// decides how a missing import should be reported.
  pub fn include_candidate_paths(self, shaders_root: &Path, import_path: &str) -> Vec<PathBuf> {
    let import_path: PathBuf = Self::path_from_xray_path(import_path);
    let mut paths: Vec<PathBuf> = vec![shaders_root.join(self.directory_name()).join(&import_path)];

    if matches!(self, Self::DirectX11) {
      paths.push(shaders_root.join(import_path));
    }

    paths
  }

  fn path_from_xray_path(path: &str) -> PathBuf {
    path
      .split(['\\', '/'])
      .filter(|component| !component.is_empty())
      .collect()
  }

  // todo: Add explicit legacy renderer variants (r1, r2).
}
