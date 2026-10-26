use crate::vfs::cache::XrayCacheKey;
use crate::vfs::cache::flight::XrayCacheFlights;

/// Ends the owner's flight however its load turns out, including an unwinding panic.
///
/// Without this a panicking load would leave its waiters blocked for the life of the process, which is the one failure
/// of coordinated loading that a caller could not diagnose from anything it can observe.
pub(crate) struct XrayCacheFlightGuard<'a> {
  pub(super) flights: &'a XrayCacheFlights,
  pub(super) key: XrayCacheKey,
}

impl Drop for XrayCacheFlightGuard<'_> {
  fn drop(&mut self) {
    self.flights.release(&self.key);
  }
}
