use std::sync::{Arc, OnceLock};

use xrf_ltx::LtxResolution;
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxResolvedIndex};
use xrf_vfs::XrayLogicalPath;

/// One entry point's resolution, and whatever has been read out of it since: each answer built once, the first that
/// lands kept where two race.
pub struct ConfigsResolvedRoot {
  pub entry: XrayLogicalPath,
  pub resolution: Arc<LtxResolution>,
  /// Whether this one can account for a value, which is what a surface reading origins has to ask for.
  pub is_explained: bool,
  /// Every section of the root, named and counted, once something has asked.
  index: OnceLock<Arc<LtxResolvedIndex>>,
  /// Everything wrong with the root, once something has asked. Verifying it walks every section it holds.
  findings: OnceLock<Arc<Vec<LtxAnchoredFinding>>>,
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
      findings: OnceLock::new(),
      index: OnceLock::new(),
      is_explained,
      resolution,
    }
  }

  /// The index, if something has already built one.
  pub fn get_index(&self) -> Option<Arc<LtxResolvedIndex>> {
    self.index.get().cloned()
  }

  /// Keeps an index built for this root, so the next ask is a lookup, answering the one kept.
  pub fn hold_index(&self, index: Arc<LtxResolvedIndex>) -> Arc<LtxResolvedIndex> {
    Arc::clone(self.index.get_or_init(|| index))
  }

  /// The findings, if something has already verified this root.
  pub fn get_findings(&self) -> Option<Arc<Vec<LtxAnchoredFinding>>> {
    self.findings.get().cloned()
  }

  /// Keeps the findings of this root, so the next ask is a lookup, answering the ones kept.
  pub fn hold_findings(&self, findings: Arc<Vec<LtxAnchoredFinding>>) -> Arc<Vec<LtxAnchoredFinding>> {
    Arc::clone(self.findings.get_or_init(|| findings))
  }
}
