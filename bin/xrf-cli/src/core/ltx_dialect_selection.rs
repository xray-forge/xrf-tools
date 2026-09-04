use std::sync::Arc;

use xrf_dltx::DltxDialect;
use xrf_ltx::{LtxDialect, LtxStandardDialect};

/// The dialect a command resolves configs under.
///
/// Standard LTX unless the caller asked for DLTX. Chosen here and nowhere else, so every command spells the choice the
/// same way and nothing downstream has to know which one is in force.
pub fn select_ltx_dialect(is_dltx: bool) -> Arc<dyn LtxDialect> {
  if is_dltx {
    Arc::new(DltxDialect)
  } else {
    Arc::new(LtxStandardDialect)
  }
}
