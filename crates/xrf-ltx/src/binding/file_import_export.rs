use std::path::Path;

use xrf_error::XrfResult;

/// A record that is read from and written to a file of its own.
pub trait FileImportExport: Sized {
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self>;

  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult;
}
