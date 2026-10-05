use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use crate::host::render_bundle::{RENDER_BUNDLE_SOURCE, RenderBundle};

/// The bundle as a test reads it: the renderer's checkout, or a folder holding none of it.
pub(crate) struct TestBundle {
  root: PathBuf,
}

impl TestBundle {
  pub(crate) fn checkout() -> Self {
    Self {
      root: PathBuf::from(RENDER_BUNDLE_SOURCE),
    }
  }

  pub(crate) fn empty() -> Self {
    Self {
      root: PathBuf::from(RENDER_BUNDLE_SOURCE).join("missing"),
    }
  }
}

impl RenderBundle for TestBundle {
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>> {
    std::fs::read(self.root.join(path))
      .map_err(|error| XrfError::new_io_error(format!("Bundled '{path}' cannot be read: {error}"), error.kind()))
  }
}
