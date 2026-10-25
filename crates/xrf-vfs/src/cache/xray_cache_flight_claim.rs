use std::sync::Arc;

use crate::cache::xray_cache_flight::XrayCacheFlight;
use crate::cache::xray_cache_flight_guard::XrayCacheFlightGuard;

/// What one caller is to a key it wants loaded.
pub(crate) enum XrayCacheFlightClaim<'a> {
  /// This caller performs the load. Dropping the guard releases the waiters, however the load ended.
  Owned(XrayCacheFlightGuard<'a>),
  /// Another thread is loading this key, and [`XrayCacheFlight::wait`] returns once it has finished.
  Waiting(Arc<XrayCacheFlight>),
  /// This thread is already loading this key further up its own stack.
  Reentrant,
}
