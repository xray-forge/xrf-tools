use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_shaders::XRayShaderSourceLoader;
use xrf_vfs::{XrayLookupScope, XrayVfs};

/// Loads shader sources through mounted sources, so an installation's `db\shaders` volume reads like a loose tree.
pub struct GamedataShaderSourceLoader<'a> {
  scope: &'a XrayLookupScope,
  vfs: &'a XrayVfs,
}

impl<'a> GamedataShaderSourceLoader<'a> {
  pub fn new(vfs: &'a XrayVfs, scope: &'a XrayLookupScope) -> Self {
    Self { scope, vfs }
  }
}

impl XRayShaderSourceLoader for GamedataShaderSourceLoader<'_> {
  fn load_source(&self, path: &Path) -> XrfResult<Option<Vec<u8>>> {
    let logical_path: &str = path.to_str().ok_or_else(|| {
      XrfError::new_read_error(format!("Shader source path is not valid unicode: {}", path.display()))
    })?;

    // Absence is not an error here: a missing include is reported by the caller as a finding against the shader that names
    // it, which is more useful than a read failure with no context.
    if self.vfs.scoped(self.scope).find(logical_path)?.is_none() {
      return Ok(None);
    }

    self.vfs.scoped(self.scope).read_bytes(logical_path).map(Some)
  }
}
