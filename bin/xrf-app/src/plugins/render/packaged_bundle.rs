use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_renderer::{RENDER_BUNDLE_SOURCE, RenderBundle};

/// The renderer's bundle as the application ships it: `resources/renderer` in its resources, or the renderer's
/// checkout where nothing was packaged, as a development build runs.
pub struct PackagedBundle {
  root: PathBuf,
}

impl PackagedBundle {
  /// Where a packaged application keeps the bundle in its resources, as `tauri.conf.json` maps it.
  pub const PACKAGED: &'static str = "resources/renderer";

  pub fn locate(resource_dir: Option<&Path>) -> Self {
    let root: PathBuf = resource_dir
      .map(|dir| dir.join(Self::PACKAGED))
      .filter(|dir| dir.is_dir())
      .unwrap_or_else(|| PathBuf::from(RENDER_BUNDLE_SOURCE));

    log::info!("Renderer bundle read from {}", root.display());

    Self { root }
  }
}

impl RenderBundle for PackagedBundle {
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>> {
    let file: PathBuf = self.root.join(path);

    std::fs::read(&file).map_err(|error| {
      XrfError::new_io_error(
        format!("Renderer bundle file '{}' cannot be read: {error}", file.display()),
        error.kind(),
      )
    })
  }
}
