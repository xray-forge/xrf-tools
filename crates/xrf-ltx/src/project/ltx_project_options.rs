use std::sync::Arc;

use crate::dialect::ltx_dialect::LtxDialect;
use crate::dialect::ltx_standard_dialect::LtxStandardDialect;

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
}

impl Default for LtxProjectOptions {
  fn default() -> Self {
    Self {
      dialect: Arc::new(LtxStandardDialect),
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
