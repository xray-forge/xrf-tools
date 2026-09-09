use std::sync::{Arc, Mutex, MutexGuard};

use xrf_ltx::LtxResolution;
use xrf_ltx_inspect::LtxResolvedIndex;
use xrf_vfs::XrayLogicalPath;

use crate::core::types::TauriResult;

/// One entry point's resolution, and whatever has been read out of it since.
///
/// The pair is what makes the cache answerable: a resolution whose entry nobody recorded cannot be checked against the
/// one being asked for.
pub struct ConfigsResolvedRoot {
  pub entry: XrayLogicalPath,
  pub resolution: Arc<LtxResolution>,
  /// Every section of the root, named and counted, once something has asked.
  index: Mutex<Option<Arc<LtxResolvedIndex>>>,
}

impl ConfigsResolvedRoot {
  pub fn new(entry: XrayLogicalPath, resolution: LtxResolution) -> Self {
    Self {
      entry,
      index: Mutex::new(None),
      resolution: Arc::new(resolution),
    }
  }

  /// The index, if something has already built one.
  pub fn get_index(&self) -> TauriResult<Option<Arc<LtxResolvedIndex>>> {
    Ok(self.index()?.clone())
  }

  /// Keeps an index built for this root, so the next ask is a lookup.
  ///
  /// Two callers racing build it twice and the later one wins; both describe the same resolution, so there is nothing
  /// to reconcile and nothing worth a second lock around the work.
  pub fn hold_index(&self, index: Arc<LtxResolvedIndex>) -> TauriResult<()> {
    *self.index()? = Some(index);

    Ok(())
  }

  fn index(&self) -> TauriResult<MutexGuard<'_, Option<Arc<LtxResolvedIndex>>>> {
    self
      .index
      .lock()
      .map_err(|error| format!("The configs index cache is unavailable: {error}"))
  }
}
