use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
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

/// The checkout's DXC library, which a test's D3D12 compiles shaders with as the application's does.
pub(crate) fn checkout_dxc() -> Option<PathBuf> {
  TestBundle::checkout().locate_bundled(GpuContext::DXC_LIBRARY)
}

impl RenderBundle for TestBundle {
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>> {
    std::fs::read(self.root.join(path))
      .map_err(|error| XrfError::new_io_error(format!("Bundled '{path}' cannot be read: {error}"), error.kind()))
  }

  fn locate_bundled(&self, path: &str) -> Option<PathBuf> {
    Some(self.root.join(path)).filter(|file| file.is_file())
  }
}
