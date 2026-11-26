use std::sync::Arc;

use xrf_vfs::{XrayRoots, XraySearch, XrayVfsHost};

use crate::LevelSourceResult;

/// The VFS host a level's files are read through, as the state every read of it takes a search from. A clone shares
/// the host.
#[derive(Clone)]
pub struct LevelMount {
  host: Arc<XrayVfsHost>,
}

impl LevelMount {
  /// A level's reads through `host`, which its own host shares with every other reader.
  pub fn new(host: Arc<XrayVfsHost>) -> Self {
    Self { host }
  }

  /// The host itself, for a reader that searches through it on its own.
  pub fn get_host(&self) -> &Arc<XrayVfsHost> {
    &self.host
  }

  /// A search over what `roots` names, mounted where it is not yet.
  ///
  /// # Errors
  ///
  /// Returns why the roots cannot be planned or mounted.
  pub fn search(&self, roots: &XrayRoots) -> LevelSourceResult<XraySearch> {
    self.host.search(roots).map_err(|error| error.to_string())
  }
}
