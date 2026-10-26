use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use crate::vfs::cache::XrayCacheKey;
use crate::vfs::cache::flight::{XrayCacheFlight, XrayCacheFlightClaim, XrayCacheFlightGuard};

/// The loads a store has in progress, so concurrent requesters of one key produce one load instead of one each.
///
/// This is coordination alone. Whether a key is worth coordinating, what a wait means for reported statistics, and what
/// a released waiter does next are the store's questions, answered in [`crate::XrayAssetCache`].
#[derive(Debug, Default)]
pub(crate) struct XrayCacheFlights {
  in_progress: Mutex<HashMap<XrayCacheKey, Arc<XrayCacheFlight>>>,
}

impl XrayCacheFlights {
  /// Says what this caller is to a key, admitting exactly one owner per key at a time.
  pub(crate) fn claim(&self, key: &XrayCacheKey) -> XrayCacheFlightClaim<'_> {
    let mut in_progress = self.lock();

    match in_progress.get(key) {
      // Waiting on a flight this thread owns would be waiting on itself, so a load that re-enters the store for the key
      // it is already loading proceeds uncoordinated rather than hanging a sweep with nothing to show for it.
      Some(flight) if flight.owner == thread::current().id() => XrayCacheFlightClaim::Reentrant,
      Some(flight) => XrayCacheFlightClaim::Waiting(Arc::clone(flight)),
      None => {
        in_progress.insert(key.clone(), Arc::new(XrayCacheFlight::new()));

        XrayCacheFlightClaim::Owned(XrayCacheFlightGuard {
          flights: self,
          key: key.clone(),
        })
      }
    }
  }

  /// Ends one flight and releases everyone waiting on it.
  pub(super) fn release(&self, key: &XrayCacheKey) {
    if let Some(flight) = self.lock().remove(key) {
      flight.finish();
    }
  }

  fn lock(&self) -> MutexGuard<'_, HashMap<XrayCacheKey, Arc<XrayCacheFlight>>> {
    self.in_progress.lock().expect("cache flights lock is never poisoned")
  }
}
