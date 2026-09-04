use std::sync::Arc;

use xrf_dltx::DltxDialect;
use xrf_ltx::{LtxDialect, LtxStandardDialect};

/// The dialect a request resolves configs under.
pub fn select_ltx_dialect(is_dltx: bool) -> Arc<dyn LtxDialect> {
  if is_dltx {
    Arc::new(DltxDialect)
  } else {
    Arc::new(LtxStandardDialect)
  }
}
