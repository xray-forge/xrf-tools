use std::sync::Arc;

use xrf_ltx::{Ltx, LtxProvenance, LtxResolution};

/// One environment config, resolved: its logical path and what it resolved to.
pub(crate) struct EnvironmentConfig {
  pub file: String,
  pub resolution: Arc<LtxResolution>,
}

impl EnvironmentConfig {
  pub fn get_ltx(&self) -> &Ltx {
    &self.resolution.ltx
  }

  /// Where each key came from, where the read recorded it.
  pub fn get_provenance(&self) -> Option<&LtxProvenance> {
    Some(&self.resolution.provenance).filter(|provenance| !provenance.is_empty())
  }
}
