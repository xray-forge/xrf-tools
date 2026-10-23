use std::sync::Arc;

use xrf_vfs::{XrayCachePolicy, XrayRoots, XraySearch, XrayVfs, XrayVfsHost};

use crate::core::types::TauriResult;

/// The application's VFS host, as the state every asset read takes a search from.
///
/// The host owns the current snapshot, mounting and the plans; this only decides what the process retains.
#[derive(Clone)]
pub struct AssetMountState {
  host: Arc<XrayVfsHost>,
}

impl AssetMountState {
  /// What the process may retain of parsed configs, across every snapshot and every project reading them, before it
  /// drops what no reader is holding: a ceiling against a session that opens several games, not a working budget.
  const CACHE_BUDGET: u64 = 512 * 1024 * 1024;

  pub fn new() -> Self {
    Self {
      host: Arc::new(XrayVfsHost::new(
        XrayVfs::new().with_cache_policy(XrayCachePolicy::configs().with_budget(Self::CACHE_BUDGET)),
      )),
    }
  }

  /// A search over what a spec names, mounted where it is not yet, a source that failed to open before staying
  /// settled: every asset read goes through here, and trying a broken volume on each would cost every read.
  pub fn search(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    self.host.search(spec).map_err(|error| error.to_string())
  }

  /// The same for a person opening the roots: planned from disk again, and every source that failed to open tried
  /// again, so a volume locked once, by a running game or a scan, opens once it is free.
  pub fn search_fresh(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    self.host.search_fresh(spec).map_err(|error| error.to_string())
  }
}
