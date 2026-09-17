use std::sync::{Arc, Mutex, MutexGuard};

use xrf_ltx::LtxResolution;
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxResolvedIndex};
use xrf_vfs::XrayLogicalPath;

use crate::core::types::TauriResult;

/// Something read out of a root once and kept beside it.
struct ConfigsHeldAnswer<T> {
  /// What this holds, for the message a caller gets when the lock is poisoned.
  subject: &'static str,
  value: Mutex<Option<Arc<T>>>,
}

impl<T> ConfigsHeldAnswer<T> {
  fn new(subject: &'static str) -> Self {
    Self {
      subject,
      value: Mutex::new(None),
    }
  }

  /// What is held, if anything has been.
  fn get(&self) -> TauriResult<Option<Arc<T>>> {
    Ok(self.locked()?.clone())
  }

  /// Keeps an answer, so the next ask is a lookup.
  fn hold(&self, value: Arc<T>) -> TauriResult<()> {
    *self.locked()? = Some(value);

    Ok(())
  }

  fn locked(&self) -> TauriResult<MutexGuard<'_, Option<Arc<T>>>> {
    self
      .value
      .lock()
      .map_err(|error| format!("The configs {} cache is unavailable: {error}", self.subject))
  }
}

/// One entry point's resolution, and whatever has been read out of it since.
pub struct ConfigsResolvedRoot {
  pub entry: XrayLogicalPath,
  pub resolution: Arc<LtxResolution>,
  /// Whether this one can account for a value, which is what a surface reading origins has to ask for.
  pub is_explained: bool,
  /// Every section of the root, named and counted, once something has asked.
  index: ConfigsHeldAnswer<LtxResolvedIndex>,
  /// Everything wrong with the root, once something has asked. Verifying it walks every section it holds.
  findings: ConfigsHeldAnswer<Vec<LtxAnchoredFinding>>,
}

impl ConfigsResolvedRoot {
  /// One resolved with every origin recorded, for a surface that has to explain a value.
  pub fn explained(entry: XrayLogicalPath, resolution: LtxResolution) -> Self {
    Self::held(entry, Arc::new(resolution), true)
  }

  /// One resolved without origins, sharing the handle the project holds rather than a copy of it.
  pub fn plain(entry: XrayLogicalPath, resolution: Arc<LtxResolution>) -> Self {
    Self::held(entry, resolution, false)
  }

  fn held(entry: XrayLogicalPath, resolution: Arc<LtxResolution>, is_explained: bool) -> Self {
    Self {
      entry,
      findings: ConfigsHeldAnswer::new("findings"),
      index: ConfigsHeldAnswer::new("index"),
      is_explained,
      resolution,
    }
  }

  /// The index, if something has already built one.
  pub fn get_index(&self) -> TauriResult<Option<Arc<LtxResolvedIndex>>> {
    self.index.get()
  }

  /// Keeps an index built for this root, so the next ask is a lookup.
  pub fn hold_index(&self, index: Arc<LtxResolvedIndex>) -> TauriResult<()> {
    self.index.hold(index)
  }

  /// The findings, if something has already verified this root.
  pub fn get_findings(&self) -> TauriResult<Option<Arc<Vec<LtxAnchoredFinding>>>> {
    self.findings.get()
  }

  /// Keeps the findings of this root, so the next ask is a lookup.
  pub fn hold_findings(&self, findings: Arc<Vec<LtxAnchoredFinding>>) -> TauriResult<()> {
    self.findings.hold(findings)
  }
}
