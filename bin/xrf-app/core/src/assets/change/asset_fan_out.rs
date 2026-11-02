use std::sync::{Arc, Mutex, PoisonError};

use xrf_vfs::XrayVfs;

use crate::assets::change::asset_change_work::AssetChangeWork;

/// One change on its way out: the snapshot the sessions move to, and what their follows put off until it is heard.
pub struct AssetFanOut {
  vfs: Arc<XrayVfs>,
  starts: Mutex<Vec<AssetChangeWork>>,
}

impl AssetFanOut {
  /// A change moving sessions to `vfs`, nothing put off yet.
  pub fn new(vfs: Arc<XrayVfs>) -> Self {
    Self {
      vfs,
      starts: Mutex::new(Vec::new()),
    }
  }

  /// The snapshot this change moves sessions to: the one it published, never a newer one, so the event naming its
  /// revision names the sessions that reached it.
  pub(crate) fn get_vfs(&self) -> &Arc<XrayVfs> {
    &self.vfs
  }

  /// Runs `start` once this change has been heard by every subscriber and page, for a follow starting work that
  /// announces a later change of its own.
  pub(crate) fn after(&self, start: impl FnOnce() + Send + 'static) {
    self
      .starts
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Box::new(start));
  }

  pub(crate) fn into_starts(self) -> Vec<AssetChangeWork> {
    self.starts.into_inner().unwrap_or_else(PoisonError::into_inner)
  }
}
