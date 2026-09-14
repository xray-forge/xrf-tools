use std::sync::Arc;

use crate::dialect::{LtxDialect, LtxStandardDialect};

/// Verification configuration.
#[derive(Clone)]
pub struct LtxProjectOptions {
  /// Whether project parsing should include schemes parsing.
  pub is_with_schemes_check: bool,
  /// Whether project parsing and checks should be stricter.
  /// Additional checks with strict mode:
  /// - Case sensitivity of include statements
  pub is_strict_check: bool,
  /// Which rules resolve this project's configs.
  pub dialect: Arc<dyn LtxDialect>,
  /// Whether a resolved root is kept for the life of the project.
  ///
  /// Off by default, because holding one is only worth it for a caller that asks for the same root again: a resolved
  /// Anomaly `system.ltx` is tens of megabytes, and a sweep reading every root once would end holding the whole tree.
  /// A long-lived session that reopens roots - the configs explorer - turns it on.
  pub is_caching_resolutions: bool,
}

impl Default for LtxProjectOptions {
  fn default() -> Self {
    Self {
      dialect: Arc::new(LtxStandardDialect),
      is_caching_resolutions: false,
      is_strict_check: false,
      is_with_schemes_check: false,
    }
  }
}

impl LtxProjectOptions {
  pub fn new() -> Self {
    Self::default()
  }

  /// The same options, resolving configs under `dialect`.
  pub fn with_dialect(mut self, dialect: Arc<dyn LtxDialect>) -> Self {
    self.dialect = dialect;

    self
  }
}
