use std::sync::Arc;

use xrf_ltx::{LtxDialect, LtxStandardDialect};

use crate::dltx_dialect::DltxDialect;

/// The dialect a caller resolves configs under, given whether they opted into DLTX.
pub fn select_ltx_dialect(is_dltx: bool) -> Arc<dyn LtxDialect> {
  if is_dltx {
    Arc::new(DltxDialect)
  } else {
    Arc::new(LtxStandardDialect)
  }
}
