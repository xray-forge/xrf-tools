use std::path::Path;

use xrf_error::XrfResult;

pub trait FileImportExport: Sized {
  fn import<P: AsRef<Path>>(path: &P) -> XrfResult<Self>;

  fn export<P: AsRef<Path>>(&self, path: &P) -> XrfResult;
}
